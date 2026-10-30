import 'package:dayflow_mobile/src/application/task_service.dart';
import 'package:dayflow_mobile/src/domain/task.dart';
import 'package:dayflow_mobile/src/repo/task_repository.dart';
import 'package:dayflow_mobile/src/home_page.dart';
import 'package:dayflow_mobile/src/platform/mobile_reminder_service.dart';
import 'package:dayflow_mobile/src/platform/mobile_task_sync.dart';
import 'package:dayflow_mobile/src/platform/task_widget_service.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

class MemoryTaskStore implements TaskStore {
  final Map<String, Task> _tasks = {};

  @override
  Future<List<Task>> getActiveTasks() async =>
      _tasks.values.where((task) => task.deletedAt == null).toList();

  @override
  Future<Task?> getTask(String id) async => _tasks[id];

  @override
  Future<void> insertTask(Task task) async {
    if (_tasks.containsKey(task.id)) throw StateError('duplicate task id');
    _tasks[task.id] = task;
  }

  @override
  Future<void> updateTask(Task task, {required int expectedVersion}) async {
    final existing = _tasks[task.id];
    if (existing == null ||
        existing.version != expectedVersion ||
        existing.deletedAt != null) {
      throw StateError('任务已被修改或删除，请刷新后重试');
    }
    _tasks[task.id] = task;
  }

  @override
  Future<void> close() async {}
}

void main() {
  group('TaskService', () {
    late MemoryTaskStore store;
    late TaskService service;

    setUp(() {
      store = MemoryTaskStore();
      service = TaskService(
        store,
        now: () => DateTime(2026, 10, 30, 12, 0),
      );
    });

    test('creates an untimed task with matching original and current schedules',
        () async {
      final task =
          await service.createTask(title: '  写周报 ', date: '2026-10-30');

      expect(task.title, '写周报');
      expect(task.scheduledDate, task.originalDate);
      expect(task.scheduledTime, isNull);
      expect(task.version, 1);
      expect((await service.loadBoard()).todayUntimed, [task]);
    });

    test('creates timed tasks and sorts today tasks by time', () async {
      final later = await service.createTask(
        title: '下午任务',
        date: '2026-10-30',
        time: '14:00',
      );
      final earlier = await service.createTask(
        title: '上午任务',
        date: '2026-10-30',
        time: '09:00',
      );

      expect((await service.loadBoard()).todayTimed, [earlier, later]);
    });

    test('keeps overdue tasks today and classifies legacy and future tasks',
        () async {
      final overdue = await service.createTask(
        title: '已逾期',
        date: '2026-10-30',
        time: '09:00',
      );
      final legacy = await service.createTask(
        title: '之前未完成',
        date: '2026-10-29',
        time: '09:00',
      );
      final future = await service.createTask(
        title: '未来任务',
        date: '2026-10-31',
      );
      final board = await service.loadBoard();

      expect(board.todayTimed, [overdue]);
      expect(board.isOverdue(overdue), isTrue);
      expect(board.legacy, [legacy]);
      expect(board.future, [future]);
    });

    test('rejects invalid titles, dates, and times', () async {
      expect(
        () => service.createTask(title: '   ', date: '2026-10-30'),
        throwsArgumentError,
      );
      expect(
        () => service.createTask(title: '任务', date: 'bad-date'),
        throwsArgumentError,
      );
      expect(
        () =>
            service.createTask(title: '任务', date: '2026-10-30', time: '25:30'),
        throwsArgumentError,
      );
    });

    test('reschedule preserves the original schedule and increments version',
        () async {
      final task = await service.createTask(
        title: '任务',
        date: '2026-10-30',
        time: '09:30',
      );
      final updated = await service.updateTask(
        task,
        title: task.title,
        date: '2026-11-02',
        time: null,
        priority: task.priority,
      );

      expect(updated.originalDate, '2026-10-30');
      expect(updated.originalTime, '09:30');
      expect(updated.scheduledDate, '2026-11-02');
      expect(updated.scheduledTime, isNull);
      expect(updated.version, 2);
    });

    test('rejects stale task writes', () async {
      final task = await service.createTask(title: '任务', date: '2026-10-30');
      await service.updateTask(
        task,
        title: '更新后的任务',
        date: task.scheduledDate,
        time: null,
        priority: task.priority,
      );

      await expectLater(
        service.completeTask(task),
        throwsStateError,
      );
    });

    test('completes, undoes, cancels, restores, and soft-deletes tasks',
        () async {
      final created = await service.createTask(title: '任务', date: '2026-10-30');
      final completed = await service.completeTask(created);
      expect(completed.status, TaskStatus.completed);
      expect(completed.completedAt, isNotNull);

      final restored = await service.undoCompletion(completed);
      expect(restored.status, TaskStatus.todo);
      expect(restored.completedAt, isNull);

      final canceled = await service.cancelTask(restored);
      final restoredCanceled = await service.restoreCanceledTask(canceled);
      expect(restoredCanceled.status, TaskStatus.todo);

      final deleted = await service.softDeleteTask(restoredCanceled);
      expect(deleted.deletedAt, isNotNull);
      expect(await service.loadBoard().then((board) => board.todayUntimed),
          isEmpty);
      expect(await store.getTask(created.id), isNotNull);
    });

    testWidgets('quick-add dialog saves a task into the local task service',
        (tester) async {
      await tester.pumpWidget(
        MaterialApp(
          home: HomePage(
            service: service,
          ),
        ),
      );
      await tester.pumpAndSettle();
      await tester.tap(find.byTooltip('添加任务'));
      await tester.pumpAndSettle();
      await tester.enterText(find.byType(TextField), '模拟器任务');
      await tester.tap(find.text('添加').last);
      await tester.pumpAndSettle();

      expect(find.text('模拟器任务'), findsOneWidget);
      expect((await service.loadBoard()).todayUntimed.single.title, '模拟器任务');
    });

    testWidgets('shows instructions for adding a home-screen widget',
        (tester) async {
      await tester.pumpWidget(
        MaterialApp(
          home: HomePage(
            service: service,
            platformSync: MobileTaskSync(
              tasks: store,
              reminders: MobileReminderService(),
              widgets: TaskWidgetService(),
            ),
          ),
        ),
      );
      await tester.pumpAndSettle();
      await tester.tap(find.byTooltip('添加桌面小组件'));
      await tester.pumpAndSettle();

      expect(find.text('添加 DayFlow 小组件'), findsOneWidget);
      expect(find.textContaining('小组件由系统管理'), findsOneWidget);
      expect(find.text('同步小组件数据'), findsOneWidget);
    });
  });
}
