import 'package:flutter_local_notifications/flutter_local_notifications.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:flutter_timezone/flutter_timezone.dart';
import 'package:integration_test/integration_test.dart';
import 'package:timezone/data/latest.dart' as tz_data;
import 'package:timezone/timezone.dart' as tz;

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();

  testWidgets('splan notification delivery probe', (tester) async {
    tz_data.initializeTimeZones();
    final timezone = await FlutterTimezone.getLocalTimezone();
    tz.setLocalLocation(tz.getLocation(timezone.identifier));

    final notifications = FlutterLocalNotificationsPlugin();
    await notifications.initialize(
      const InitializationSettings(
        android: AndroidInitializationSettings('@mipmap/ic_launcher'),
      ),
    );
    await notifications
        .resolvePlatformSpecificImplementation<
            AndroidFlutterLocalNotificationsPlugin>()
        ?.createNotificationChannel(
          const AndroidNotificationChannel(
            'splan_probe',
            'Splan probe',
            description: 'probe',
            importance: Importance.high,
          ),
        );

    await notifications.show(
      999998,
      'Splan probe immediate',
      'immediate delivery probe',
      const NotificationDetails(
        android: AndroidNotificationDetails(
          'splan_probe',
          'Splan probe',
          channelDescription: 'probe',
          importance: Importance.high,
        ),
      ),
    );

    final fireAt = tz.TZDateTime.from(
      DateTime.now().add(const Duration(seconds: 20)),
      tz.local,
    );
    await notifications.zonedSchedule(
      999999,
      'Splan probe',
      'notification delivery probe',
      fireAt,
      const NotificationDetails(
        android: AndroidNotificationDetails(
          'splan_probe',
          'Splan probe',
          channelDescription: 'probe',
          importance: Importance.high,
        ),
      ),
      androidScheduleMode: AndroidScheduleMode.inexactAllowWhileIdle,
      uiLocalNotificationDateInterpretation:
          UILocalNotificationDateInterpretation.absoluteTime,
    );

    await Future<void>.delayed(const Duration(seconds: 35));
  });
}
