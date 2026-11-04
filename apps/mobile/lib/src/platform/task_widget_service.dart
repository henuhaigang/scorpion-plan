import 'dart:convert';

import 'package:home_widget/home_widget.dart';

import '../domain/task.dart';
import '../domain/task_board.dart';

const splanAppGroupId = 'group.com.splan.splanMobile';
const splanWidgetKind = 'SplanWidget';
const splanAndroidWidgetClass =
    'com.splan.splan_mobile.SplanWidgetProvider';

class TaskWidgetSnapshot {
  final List<Task> today;
  final int legacyCount;
  final List<Task> legacy;

  const TaskWidgetSnapshot({
    required this.today,
    required this.legacyCount,
    required this.legacy,
  });

  factory TaskWidgetSnapshot.fromTasks(List<Task> tasks, DateTime now) {
    final board = TaskBoard.fromTasks(tasks, now);
    return TaskWidgetSnapshot(
      today: [...board.todayTimed, ...board.todayUntimed].take(4).toList(),
      legacyCount: board.legacy.length,
      legacy: board.legacy.take(3).toList(),
    );
  }

  Map<String, Object?> toJson() => {
        'today': today.map(_taskJson).toList(),
        'legacyCount': legacyCount,
        'legacy': legacy.map(_taskJson).toList(),
      };

  String encode() => jsonEncode(toJson());

  static Map<String, Object?> _taskJson(Task task) => {
        'id': task.id,
        'title': task.title,
        'time': task.scheduledTime,
      };
}

class TaskWidgetService {
  final DateTime Function() _now;

  TaskWidgetService({DateTime Function()? now}) : _now = now ?? DateTime.now;

  Future<void> initialize() async {
    await HomeWidget.setAppGroupId(splanAppGroupId);
  }

  Future<void> publish(List<Task> tasks) async {
    await initialize();
    final snapshot = TaskWidgetSnapshot.fromTasks(tasks, _now());
    await HomeWidget.saveWidgetData<String>(
      'splanTasks',
      snapshot.encode(),
    );
    await HomeWidget.updateWidget(
      name: splanWidgetKind,
      iOSName: splanWidgetKind,
      qualifiedAndroidName: splanAndroidWidgetClass,
    );
  }
}
