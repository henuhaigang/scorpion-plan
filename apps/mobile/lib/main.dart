import 'dart:async';

import 'package:home_widget/home_widget.dart';

import 'package:flutter/material.dart';
import 'src/app.dart';
import 'src/platform/task_widget_service.dart';
import 'src/platform/widget_background_callback.dart';

void main() {
  WidgetsFlutterBinding.ensureInitialized();
  runApp(const DayFlowApp());
  unawaited(_initializeWidgetIntegration());
}

Future<void> _initializeWidgetIntegration() async {
  try {
    await HomeWidget.setAppGroupId(dayFlowAppGroupId);
    await HomeWidget.registerInteractivityCallback(handleDayFlowWidgetAction);
  } catch (error, stackTrace) {
    debugPrint('DayFlow widget initialization failed: $error\n$stackTrace');
  }
}
