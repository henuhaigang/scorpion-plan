import 'package:dayflow_mobile/src/application/task_service.dart';
import 'package:dayflow_mobile/src/platform/mobile_reminder_service.dart';
import 'package:dayflow_mobile/src/platform/mobile_task_sync.dart';
import 'package:dayflow_mobile/src/platform/task_widget_service.dart';
import 'package:dayflow_mobile/src/repo/task_repository.dart';
import 'package:flutter_local_notifications/flutter_local_notifications.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();

  testWidgets('task persists when notification permission is denied',
      (tester) async {
    final repository = TaskRepository();
    final tasks = TaskService(repository);
    final reminders = MobileReminderService();
    final date = DateTime.now().add(const Duration(days: 1));
    final task = await tasks.createTask(
      title: 'permission_denied_${DateTime.now().microsecondsSinceEpoch}',
      date: _dateString(date),
      time: '23:55',
    );
    final notificationId = MobileReminderService.notificationId(task.id);

    final issue = await MobileTaskSync(
      tasks: repository,
      reminders: reminders,
      widgets: TaskWidgetService(),
    ).synchronize();

    expect(issue, contains('通知'));
    expect(
      (await repository.getActiveTasks()).any((saved) => saved.id == task.id),
      isTrue,
    );
    final pending =
        await FlutterLocalNotificationsPlugin().pendingNotificationRequests();
    expect(pending.any((notification) => notification.id == notificationId),
        isFalse);

    await tasks.softDeleteTask(task);
    await repository.close();
  });
}

String _dateString(DateTime date) => '${date.year.toString().padLeft(4, '0')}-'
    '${date.month.toString().padLeft(2, '0')}-'
    '${date.day.toString().padLeft(2, '0')}';
