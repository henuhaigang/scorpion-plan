import 'dart:io';

import 'package:dayflow_mobile/src/application/task_service.dart';
import 'package:dayflow_mobile/src/platform/mobile_reminder_service.dart';
import 'package:dayflow_mobile/src/repo/task_repository.dart';
import 'package:flutter_local_notifications/flutter_local_notifications.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();

  testWidgets('timed task reminders are scheduled and canceled by lifecycle',
      (tester) async {
    final repository = TaskRepository();
    final tasks = TaskService(repository);
    final reminders = MobileReminderService();
    await reminders.initialize();

    final hasPermission = await reminders.hasPermission();
    if (Platform.isIOS && !hasPermission) {
      final date = DateTime.now().add(const Duration(days: 1));
      final task = await tasks.createTask(
        title: 'permission_denied_${DateTime.now().microsecondsSinceEpoch}',
        date: _dateString(date),
        time: '23:55',
      );
      await reminders.reconcile([task]);
      final pending =
          await FlutterLocalNotificationsPlugin().pendingNotificationRequests();
      expect(
        pending.any(
          (notification) =>
              notification.id == MobileReminderService.notificationId(task.id),
        ),
        isFalse,
      );
      await tasks.softDeleteTask(task);
      await repository.close();
      return;
    }

    if (!hasPermission) {
      expect(await reminders.requestPermission(), isTrue);
    }
    final date = DateTime.now().add(const Duration(days: 1));
    final task = await tasks.createTask(
      title: 'reminder_${DateTime.now().microsecondsSinceEpoch}',
      date: _dateString(date),
      time: '23:55',
    );
    final notificationId = MobileReminderService.notificationId(task.id);

    await reminders.reconcile([task]);
    var pending =
        await FlutterLocalNotificationsPlugin().pendingNotificationRequests();
    expect(pending.any((notification) => notification.id == notificationId),
        isTrue);

    final completed = await tasks.completeTask(task);
    await reminders.reconcile([completed]);
    pending =
        await FlutterLocalNotificationsPlugin().pendingNotificationRequests();
    expect(pending.any((notification) => notification.id == notificationId),
        isFalse);

    await tasks.softDeleteTask(completed);
    await repository.close();
  });
}

String _dateString(DateTime date) => '${date.year.toString().padLeft(4, '0')}-'
    '${date.month.toString().padLeft(2, '0')}-'
    '${date.day.toString().padLeft(2, '0')}';
