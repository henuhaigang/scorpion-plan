/// Task status enum matching the desktop domain.
enum TaskStatus { todo, completed, canceled }

/// Task priority enum matching the desktop domain.
enum TaskPriority { low, normal, high }

/// Reminder preference enum matching the desktop domain.
enum ReminderPreference { inheritDefault, custom, disabled }

/// A task entity matching the desktop domain model.
class Task {
  final String id;
  final String title;
  final TaskStatus status;
  final TaskPriority priority;
  final String? scheduledDate; // YYYY-MM-DD
  final String? scheduledTime; // HH:MM
  final bool isAllDay;
  final ReminderPreference reminderMode;
  final int? reminderMinutesBefore;
  final String createdAt;
  final String updatedAt;
  final int version;

  const Task({
    required this.id,
    required this.title,
    required this.status,
    required this.priority,
    this.scheduledDate,
    this.scheduledTime,
    required this.isAllDay,
    required this.reminderMode,
    this.reminderMinutesBefore,
    required this.createdAt,
    required this.updatedAt,
    required this.version,
  });

  factory Task.fromJson(Map<String, dynamic> json) {
    return Task(
      id: json['id'] as String,
      title: json['title'] as String,
      status: TaskStatus.values.byName(json['status'] as String),
      priority: TaskPriority.values.byName(json['priority'] as String),
      scheduledDate: json['scheduled_date'] as String?,
      scheduledTime: json['scheduled_time'] as String?,
      isAllDay: json['is_all_day'] as bool? ?? false,
      reminderMode: ReminderPreference.values.byName(
        json['reminder_mode'] as String? ?? 'inheritDefault',
      ),
      reminderMinutesBefore: json['reminder_minutes_before'] as int?,
      createdAt: json['created_at'] as String,
      updatedAt: json['updated_at'] as String,
      version: json['version'] as int? ?? 1,
    );
  }

  Map<String, dynamic> toJson() {
    return {
      'id': id,
      'title': title,
      'status': status.name,
      'priority': priority.name,
      'scheduled_date': scheduledDate,
      'scheduled_time': scheduledTime,
      'is_all_day': isAllDay,
      'reminder_mode': reminderMode.name,
      'reminder_minutes_before': reminderMinutesBefore,
      'created_at': createdAt,
      'updated_at': updatedAt,
      'version': version,
    };
  }
}
