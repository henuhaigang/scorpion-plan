import 'dart:io';

import 'package:flutter_local_notifications/flutter_local_notifications.dart';
import 'package:flutter_timezone/flutter_timezone.dart';
import 'package:timezone/data/latest.dart' as tz_data;
import 'package:timezone/timezone.dart' as tz;

import '../domain/task.dart';

class MobileReminderService {
  final FlutterLocalNotificationsPlugin _notifications;
  final DateTime Function() _now;
  bool _initialized = false;
  String? _timezoneIdentifier;

  MobileReminderService({
    FlutterLocalNotificationsPlugin? notifications,
    DateTime Function()? now,
  })  : _notifications = notifications ?? FlutterLocalNotificationsPlugin(),
        _now = now ?? DateTime.now;

  Future<void> initialize() async {
    tz_data.initializeTimeZones();
    final timezone = await FlutterTimezone.getLocalTimezone();
    if (_timezoneIdentifier != timezone.identifier) {
      tz.setLocalLocation(tz.getLocation(timezone.identifier));
      _timezoneIdentifier = timezone.identifier;
    }
    if (_initialized) return;

    await _notifications.initialize(
      const InitializationSettings(
        android: AndroidInitializationSettings('@mipmap/ic_launcher'),
        iOS: DarwinInitializationSettings(
          requestAlertPermission: false,
          requestBadgePermission: false,
          requestSoundPermission: false,
        ),
      ),
    );
    await _notifications
        .resolvePlatformSpecificImplementation<
            AndroidFlutterLocalNotificationsPlugin>()
        ?.createNotificationChannel(
          const AndroidNotificationChannel(
            'task_reminders',
            '任务提醒',
            description: '按任务计划时间发送提醒',
            importance: Importance.high,
          ),
        );
    _initialized = true;
  }

  Future<bool> hasPermission() async {
    await initialize();
    if (Platform.isAndroid) {
      return await _notifications
              .resolvePlatformSpecificImplementation<
                  AndroidFlutterLocalNotificationsPlugin>()
              ?.areNotificationsEnabled() ??
          false;
    }
    if (Platform.isIOS) {
      final status = await _notifications
          .resolvePlatformSpecificImplementation<
              IOSFlutterLocalNotificationsPlugin>()
          ?.checkPermissions();
      return status?.isEnabled ?? false;
    }
    return false;
  }

  Future<bool> requestPermission() async {
    await initialize();
    if (Platform.isAndroid) {
      return await _notifications
              .resolvePlatformSpecificImplementation<
                  AndroidFlutterLocalNotificationsPlugin>()
              ?.requestNotificationsPermission() ??
          false;
    }
    if (Platform.isIOS) {
      return await _notifications
              .resolvePlatformSpecificImplementation<
                  IOSFlutterLocalNotificationsPlugin>()
              ?.requestPermissions(alert: true, sound: true) ??
          false;
    }
    return false;
  }

  Future<void> reconcile(List<Task> tasks) async {
    await initialize();
    final allowed = await hasPermission();
    final desired = <int, (Task, DateTime)>{};
    if (allowed) {
      for (final task in tasks) {
        final fireAt = reminderFireAt(task, _now());
        if (fireAt != null) desired[notificationId(task.id)] = (task, fireAt);
      }
    }

    final pending = await _notifications.pendingNotificationRequests();
    for (final notification in pending) {
      if (!desired.containsKey(notification.id)) {
        await _notifications.cancel(notification.id);
      }
    }

    for (final entry in desired.entries) {
      final (task, fireAt) = entry.value;
      await _notifications.cancel(entry.key);
      await _notifications.zonedSchedule(
        entry.key,
        '任务提醒',
        task.title,
        tz.TZDateTime.from(fireAt, tz.local),
        const NotificationDetails(
          android: AndroidNotificationDetails(
            'task_reminders',
            '任务提醒',
            channelDescription: '按任务计划时间发送提醒',
            importance: Importance.high,
            priority: Priority.high,
          ),
          iOS: DarwinNotificationDetails(),
        ),
        androidScheduleMode: AndroidScheduleMode.inexactAllowWhileIdle,
        uiLocalNotificationDateInterpretation:
            UILocalNotificationDateInterpretation.absoluteTime,
        payload: task.id,
      );
    }
  }

  static DateTime? reminderFireAt(Task task, DateTime now) {
    if (task.status != TaskStatus.todo ||
        task.deletedAt != null ||
        task.scheduledTime == null ||
        task.reminderMode == ReminderPreference.disabled) {
      return null;
    }

    final parts = task.scheduledTime!.split(':').map(int.parse).toList();
    final scheduled = DateTime(
      int.parse(task.scheduledDate.substring(0, 4)),
      int.parse(task.scheduledDate.substring(5, 7)),
      int.parse(task.scheduledDate.substring(8, 10)),
      parts[0],
      parts[1],
    );
    final lead = switch (task.reminderMode) {
      ReminderPreference.inheritDefault => 10,
      ReminderPreference.custom => task.reminderMinutesBefore ??
          (throw ArgumentError('Custom reminder requires a lead time.')),
      ReminderPreference.disabled => 0,
    };
    if (lead < 0) throw ArgumentError('Reminder lead time cannot be negative.');
    final fireAt = scheduled.subtract(Duration(minutes: lead));
    return fireAt.isAfter(now) ? fireAt : null;
  }

  static int notificationId(String taskId) {
    var hash = 0x811c9dc5;
    for (final codeUnit in taskId.codeUnits) {
      hash = ((hash ^ codeUnit) * 0x01000193) & 0x7fffffff;
    }
    return hash == 0 ? 1 : hash;
  }
}
