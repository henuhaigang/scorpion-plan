import 'package:dayflow_mobile/src/domain/task.dart';
import 'package:dayflow_mobile/src/platform/mobile_reminder_service.dart';
import 'package:dayflow_mobile/src/platform/task_widget_service.dart';
import 'package:flutter_test/flutter_test.dart';

Task _task({
  String id = 'task-1',
  String date = '2026-10-30',
  String? time = '12:00',
  TaskStatus status = TaskStatus.todo,
  ReminderPreference reminderMode = ReminderPreference.inheritDefault,
  int? minutesBefore,
}) =>
    Task(
      id: id,
      title: id,
      status: status,
      priority: TaskPriority.normal,
      originalDate: date,
      originalTime: time,
      originalIsAllDay: false,
      scheduledDate: date,
      scheduledTime: time,
      isAllDay: false,
      reminderMode: reminderMode,
      reminderMinutesBefore: minutesBefore,
      createdAt: '2026-10-30T00:00:00Z',
      updatedAt: '2026-10-30T00:00:00Z',
      completedAt: null,
      deletedAt: null,
      version: 1,
    );

void main() {
  group('Mobile reminders', () {
    final now = DateTime(2026, 10, 30, 10);

    test('uses the 10-minute inherited default', () {
      expect(
        MobileReminderService.reminderFireAt(_task(), now),
        DateTime(2026, 10, 30, 11, 50),
      );
    });

    test('honors custom and disabled reminder preferences', () {
      expect(
        MobileReminderService.reminderFireAt(
          _task(
            reminderMode: ReminderPreference.custom,
            minutesBefore: 30,
          ),
          now,
        ),
        DateTime(2026, 10, 30, 11, 30),
      );
      expect(
        MobileReminderService.reminderFireAt(
          _task(reminderMode: ReminderPreference.disabled),
          now,
        ),
        isNull,
      );
    });

    test('rejects invalid custom reminder lead times', () {
      expect(
        () => MobileReminderService.reminderFireAt(
          _task(reminderMode: ReminderPreference.custom),
          now,
        ),
        throwsArgumentError,
      );
      expect(
        () => MobileReminderService.reminderFireAt(
          _task(
            reminderMode: ReminderPreference.custom,
            minutesBefore: -1,
          ),
          now,
        ),
        throwsArgumentError,
      );
    });

    test('does not schedule untimed, terminal, or past reminders', () {
      expect(
          MobileReminderService.reminderFireAt(_task(time: null), now), isNull);
      expect(
        MobileReminderService.reminderFireAt(
          _task(status: TaskStatus.completed),
          now,
        ),
        isNull,
      );
      expect(
        MobileReminderService.reminderFireAt(
          _task(time: '10:05'),
          now,
        ),
        isNull,
      );
    });

    test('generates stable positive notification identifiers', () {
      final first = MobileReminderService.notificationId('stable-task-id');
      expect(first, greaterThan(0));
      expect(first, MobileReminderService.notificationId('stable-task-id'));
      expect(first, isNot(MobileReminderService.notificationId('other-task')));
    });
  });

  test('widget snapshot contains today tasks and legacy summaries', () {
    final today = _task(id: 'today');
    final legacy = _task(id: 'legacy', date: '2026-10-29');
    final snapshot = TaskWidgetSnapshot.fromTasks(
      [today, legacy],
      DateTime(2026, 10, 30, 10),
    );

    expect(snapshot.today.single.id, 'today');
    expect(snapshot.legacyCount, 1);
    expect(snapshot.legacy.single.id, 'legacy');
    expect(snapshot.toJson()['today'], [
      {'id': 'today', 'title': 'today', 'time': '12:00'},
    ]);

    final untimedSnapshot = TaskWidgetSnapshot.fromTasks(
      [_task(id: 'untimed', time: null)],
      DateTime(2026, 10, 30, 10),
    );
    expect(untimedSnapshot.toJson()['today'], [
      {'id': 'untimed', 'title': 'untimed', 'time': null},
    ]);
  });
}
