import 'package:uuid/uuid.dart';

import '../domain/task.dart';
import '../domain/task_board.dart';
import '../repo/task_repository.dart';

class TaskService {
  final TaskStore _tasks;
  final DateTime Function() _now;
  final Uuid _uuid;

  TaskService(
    this._tasks, {
    DateTime Function()? now,
    Uuid? uuid,
  })  : _now = now ?? DateTime.now,
        _uuid = uuid ?? const Uuid();

  Future<TaskBoard> loadBoard() async =>
      TaskBoard.fromTasks(await _tasks.getActiveTasks(), _now());

  String todayDate() => _formatDate(_now());

  Future<List<Task>> loadHistory() async {
    final tasks = await _tasks.getActiveTasks();
    return tasks.where((task) => task.status != TaskStatus.todo).toList()
      ..sort((a, b) => b.updatedAt.compareTo(a.updatedAt));
  }

  Future<Task> createTask({
    required String title,
    required String date,
    String? time,
    TaskPriority priority = TaskPriority.normal,
  }) async {
    final cleanTitle = _validateTitle(title);
    _validateSchedule(date, time);
    final now = _timestamp();
    final task = Task(
      id: _uuid.v4(),
      title: cleanTitle,
      status: TaskStatus.todo,
      priority: priority,
      originalDate: date,
      originalTime: time,
      originalIsAllDay: false,
      scheduledDate: date,
      scheduledTime: time,
      isAllDay: false,
      reminderMode: ReminderPreference.inheritDefault,
      reminderMinutesBefore: null,
      createdAt: now,
      updatedAt: now,
      completedAt: null,
      deletedAt: null,
      version: 1,
    );
    await _tasks.insertTask(task);
    return task;
  }

  Future<Task> updateTask(
    Task task, {
    required String title,
    required String date,
    required String? time,
    required TaskPriority priority,
  }) async {
    _ensureTodo(task);
    final cleanTitle = _validateTitle(title);
    _validateSchedule(date, time);
    return _save(task.copyWith(
      title: cleanTitle,
      priority: priority,
      scheduledDate: date,
      scheduledTime: time,
      clearScheduledTime: time == null,
    ));
  }

  Future<Task> completeTask(Task task) async {
    _ensureTodo(task);
    return _save(task.copyWith(
      status: TaskStatus.completed,
      completedAt: _timestamp(),
    ));
  }

  Future<Task> undoCompletion(Task task) async {
    if (task.status != TaskStatus.completed || task.deletedAt != null) {
      throw StateError('Only completed tasks can be restored.');
    }
    return _save(task.copyWith(
      status: TaskStatus.todo,
      clearCompletedAt: true,
    ));
  }

  Future<Task> cancelTask(Task task) async {
    _ensureTodo(task);
    return _save(task.copyWith(status: TaskStatus.canceled));
  }

  Future<Task> restoreCanceledTask(Task task) async {
    if (task.status != TaskStatus.canceled || task.deletedAt != null) {
      throw StateError('Only canceled tasks can be restored.');
    }
    return _save(task.copyWith(status: TaskStatus.todo));
  }

  Future<Task> softDeleteTask(Task task) async {
    if (task.deletedAt != null) {
      throw StateError('Task is already deleted.');
    }
    return _save(task.copyWith(deletedAt: _timestamp()));
  }

  Future<Task> _save(Task task) async {
    final updated = Task(
      id: task.id,
      title: task.title,
      status: task.status,
      priority: task.priority,
      originalDate: task.originalDate,
      originalTime: task.originalTime,
      originalIsAllDay: task.originalIsAllDay,
      scheduledDate: task.scheduledDate,
      scheduledTime: task.scheduledTime,
      isAllDay: task.isAllDay,
      reminderMode: task.reminderMode,
      reminderMinutesBefore: task.reminderMinutesBefore,
      createdAt: task.createdAt,
      updatedAt: _timestamp(),
      completedAt: task.completedAt,
      deletedAt: task.deletedAt,
      version: task.version + 1,
    );
    await _tasks.updateTask(updated, expectedVersion: task.version);
    return updated;
  }

  String _timestamp() => _now().toUtc().toIso8601String();

  String _validateTitle(String value) {
    final title = value.trim();
    final length = title.runes.length;
    if (length == 0) throw ArgumentError('请输入任务内容');
    if (length > 200) throw ArgumentError('任务标题不能超过 200 个字符');
    return title;
  }

  void _validateSchedule(String date, String? time) {
    try {
      final parsedDate = DateTime.parse(date);
      if (_formatDate(parsedDate) != date) throw const FormatException();
    } on FormatException {
      throw ArgumentError('日期无效');
    }
    if (time != null) {
      final match = RegExp(r'^([01]\d|2[0-3]):([0-5]\d)$').firstMatch(time);
      if (match == null) throw ArgumentError('时间无效，请使用 24 小时制');
    }
  }

  String _formatDate(DateTime date) =>
      '${date.year.toString().padLeft(4, '0')}-'
      '${date.month.toString().padLeft(2, '0')}-'
      '${date.day.toString().padLeft(2, '0')}';

  void _ensureTodo(Task task) {
    if (task.status != TaskStatus.todo || task.deletedAt != null) {
      throw StateError('Only active tasks can be changed.');
    }
  }
}
