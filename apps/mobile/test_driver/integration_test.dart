import 'dart:io';

import 'package:integration_test/integration_test_driver_extended.dart';

Future<void> main() async {
  await integrationDriver(
    onScreenshot: (screenshotName, screenshotBytes, [args]) async {
      final directory = Directory(
        Platform.environment['SPLAN_SCREENSHOT_DIR'] ??
            Directory.systemTemp.path,
      );
      await directory.create(recursive: true);
      await File('${directory.path}/$screenshotName.png')
          .writeAsBytes(screenshotBytes);
      return true;
    },
  );
}
