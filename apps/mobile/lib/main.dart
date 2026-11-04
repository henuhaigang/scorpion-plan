import 'dart:async';

import 'package:home_widget/home_widget.dart';

import 'package:flutter/material.dart';
import 'src/app.dart';
import 'src/platform/task_widget_service.dart';
import 'src/platform/widget_background_callback.dart';

void main() {
  WidgetsFlutterBinding.ensureInitialized();
  runApp(const SplanApp());
  unawaited(_initializeWidgetIntegration());
}

Future<void> _initializeWidgetIntegration() async {
  try {
    await HomeWidget.setAppGroupId(splanAppGroupId);
    await HomeWidget.registerInteractivityCallback(handleSplanWidgetAction);
  } catch (error, stackTrace) {
    debugPrint('Splan widget initialization failed: $error\n$stackTrace');
  }
}
