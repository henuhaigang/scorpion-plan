import 'package:flutter/widgets.dart';

import '../application/task_service.dart';
import '../domain/task.dart';
import '../repo/task_repository.dart';
import 'mobile_reminder_service.dart';
import 'mobile_task_sync.dart';
import 'task_widget_service.dart';

@pragma('vm:entry-point')
Future<void> handleDayFlowWidgetAction(Uri? uri) async {
  if (uri?.host != 'complete') return;
  final taskId = uri?.queryParameters['id'];
  if (taskId == null || taskId.isEmpty) {
    throw ArgumentError('Widget completion action requires a task id.');
  }

  WidgetsFlutterBinding.ensureInitialized();
  // sqflite can share a path-cached handle with the foreground repository.
  final tasks = TaskRepository();
  final task = await tasks.getTask(taskId);
  if (task != null &&
      task.status == TaskStatus.todo &&
      task.deletedAt == null) {
    await TaskService(tasks).completeTask(task);
  }

  final sync = MobileTaskSync(
    tasks: tasks,
    reminders: MobileReminderService(),
    widgets: TaskWidgetService(),
  );
  final issue = await sync.synchronize();
  if (issue != null) debugPrint('DayFlow widget action: $issue');
}
