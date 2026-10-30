import 'dart:convert';

import 'package:home_widget/home_widget.dart';

import '../application/task_service.dart';
import '../domain/task.dart';
import '../repo/task_repository.dart';
import 'mobile_reminder_service.dart';
import 'task_widget_service.dart';

class MobileTaskSync {
  final TaskStore _tasks;
  final MobileReminderService _reminders;
  final TaskWidgetService _widgets;

  MobileTaskSync({
    required TaskStore tasks,
    required MobileReminderService reminders,
    required TaskWidgetService widgets,
  })  : _tasks = tasks,
        _reminders = reminders,
        _widgets = widgets;

  Future<String?> synchronize({
    bool requestNotificationPermission = false,
  }) async {
    String? issue;
    try {
      await _applyPendingWidgetCompletions();
      final tasks = await _tasks.getActiveTasks();
      if (requestNotificationPermission) {
        final enabled = await _reminders.hasPermission() ||
            await _reminders.requestPermission();
        if (!enabled) {
          issue = '任务已保存，但通知权限未开启。请在系统设置中允许 DayFlow 发送通知。';
        }
      } else if (_hasUpcomingReminder(tasks) &&
          !await _reminders.hasPermission()) {
        issue = '定时任务提醒未开启。请允许 DayFlow 发送通知，以接收提前 10 分钟的提醒。';
      }

      try {
        await _reminders.reconcile(tasks);
      } catch (error) {
        issue = '任务已保存，但提醒安排失败：$error';
      }
      await _widgets.publish(tasks);
    } catch (error) {
      issue = '任务已保存，但无法更新提醒或小组件：$error';
    }
    return issue;
  }

  bool _hasUpcomingReminder(List<Task> tasks) => tasks.any(
        (task) =>
            task.status == TaskStatus.todo &&
            task.deletedAt == null &&
            MobileReminderService.reminderFireAt(task, DateTime.now()) != null,
      );

  Future<void> _applyPendingWidgetCompletions() async {
    final raw = await HomeWidget.getWidgetData<String>(
      'pendingCompletionIds',
      defaultValue: '[]',
    );
    if (raw == null || raw.isEmpty) return;
    final ids = (jsonDecode(raw) as List<dynamic>).cast<String>();
    for (final id in ids) {
      final task = await _tasks.getTask(id);
      if (task != null &&
          task.status == TaskStatus.todo &&
          task.deletedAt == null) {
        await TaskService(_tasks).completeTask(task);
      }
    }
    await HomeWidget.saveWidgetData<String>('pendingCompletionIds', '[]');
  }
}
