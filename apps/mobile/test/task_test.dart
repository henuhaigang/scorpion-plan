import 'package:flutter_test/flutter_test.dart';
import 'package:dayflow_mobile/src/domain/task.dart';

void main() {
  group('Task model', () {
    test('fromJson parses valid task', () {
      final json = {
        'id': 'test-id',
        'title': 'Test Task',
        'status': 'todo',
        'priority': 'normal',
        'original_date': '2026-10-26',
        'original_time': '10:00',
        'original_is_all_day': false,
        'scheduled_date': '2026-10-26',
        'scheduled_time': '10:00',
        'is_all_day': false,
        'reminder_mode': 'inheritDefault',
        'reminder_minutes_before': null,
        'created_at': '2026-10-26T00:00:00Z',
        'updated_at': '2026-10-26T00:00:00Z',
        'completed_at': null,
        'deleted_at': null,
        'version': 1,
      };
      final task = Task.fromJson(json);
      expect(task.id, 'test-id');
      expect(task.title, 'Test Task');
      expect(task.status, TaskStatus.todo);
      expect(task.priority, TaskPriority.normal);
      expect(task.scheduledDate, '2026-10-26');
      expect(task.scheduledTime, '10:00');
      expect(task.originalDate, '2026-10-26');
      expect(task.isAllDay, false);
      expect(task.reminderMode, ReminderPreference.inheritDefault);
      expect(task.version, 1);
    });

    test('toJson produces valid map', () {
      const task = Task(
        id: 'test-id',
        title: 'Test Task',
        status: TaskStatus.todo,
        priority: TaskPriority.high,
        originalDate: '2026-10-26',
        originalTime: '14:30',
        originalIsAllDay: false,
        scheduledDate: '2026-10-26',
        scheduledTime: '14:30',
        isAllDay: false,
        reminderMode: ReminderPreference.custom,
        reminderMinutesBefore: 15,
        createdAt: '2026-10-26T00:00:00Z',
        updatedAt: '2026-10-26T00:00:00Z',
        completedAt: null,
        deletedAt: null,
        version: 2,
      );
      final json = task.toJson();
      expect(json['id'], 'test-id');
      expect(json['title'], 'Test Task');
      expect(json['status'], 'todo');
      expect(json['priority'], 'high');
      expect(json['scheduled_date'], '2026-10-26');
      expect(json['scheduled_time'], '14:30');
      expect(json['original_date'], '2026-10-26');
      expect(json['is_all_day'], 0);
      expect(json['reminder_mode'], 'custom');
      expect(json['reminder_minutes_before'], 15);
      expect(json['version'], 2);
    });

    test('round-trip preserves all fields', () {
      const original = Task(
        id: 'round-trip-id',
        title: 'Round Trip',
        status: TaskStatus.completed,
        priority: TaskPriority.low,
        originalDate: '2026-10-26',
        originalTime: null,
        originalIsAllDay: true,
        scheduledDate: '2026-10-26',
        scheduledTime: null,
        isAllDay: true,
        reminderMode: ReminderPreference.disabled,
        reminderMinutesBefore: null,
        createdAt: '2026-10-26T00:00:00Z',
        updatedAt: '2026-10-26T00:00:00Z',
        completedAt: '2026-10-27T00:00:00Z',
        deletedAt: null,
        version: 5,
      );
      final restored = Task.fromJson(original.toJson());
      expect(restored.id, original.id);
      expect(restored.title, original.title);
      expect(restored.status, original.status);
      expect(restored.priority, original.priority);
      expect(restored.scheduledDate, original.scheduledDate);
      expect(restored.scheduledTime, original.scheduledTime);
      expect(restored.originalDate, original.originalDate);
      expect(restored.originalIsAllDay, original.originalIsAllDay);
      expect(restored.isAllDay, original.isAllDay);
      expect(restored.reminderMode, original.reminderMode);
      expect(restored.reminderMinutesBefore, original.reminderMinutesBefore);
      expect(restored.version, original.version);
    });
  });
}
