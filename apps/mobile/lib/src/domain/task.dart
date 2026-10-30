enum TaskStatus { todo, completed, canceled }

enum TaskPriority { low, normal, high }

enum ReminderPreference { inheritDefault, custom, disabled }

class Task {
  final String id;
  final String title;
  final TaskStatus status;
  final TaskPriority priority;
  final String originalDate;
  final String? originalTime;
  final bool originalIsAllDay;
  final String scheduledDate;
  final String? scheduledTime;
  final bool isAllDay;
  final ReminderPreference reminderMode;
  final int? reminderMinutesBefore;
  final String createdAt;
  final String updatedAt;
  final String? completedAt;
  final String? deletedAt;
  final int version;

  const Task({
    required this.id,
    required this.title,
    required this.status,
    required this.priority,
    required this.originalDate,
    required this.originalTime,
    required this.originalIsAllDay,
    required this.scheduledDate,
    required this.scheduledTime,
    required this.isAllDay,
    required this.reminderMode,
    required this.reminderMinutesBefore,
    required this.createdAt,
    required this.updatedAt,
    required this.completedAt,
    required this.deletedAt,
    required this.version,
  });

  factory Task.fromJson(Map<String, dynamic> json) {
    bool readBool(Object? value) => value == true || value == 1;
    T enumValue<T extends Enum>(List<T> values, String name) =>
        values.firstWhere(
          (value) => value.name.toLowerCase() == name.toLowerCase(),
        );

    return Task(
      id: json['id'] as String,
      title: json['title'] as String,
      status: enumValue(TaskStatus.values, json['status'] as String),
      priority: enumValue(TaskPriority.values, json['priority'] as String),
      originalDate: json['original_date'] as String,
      originalTime: json['original_time'] as String?,
      originalIsAllDay: readBool(json['original_is_all_day']),
      scheduledDate: json['scheduled_date'] as String,
      scheduledTime: json['scheduled_time'] as String?,
      isAllDay: readBool(json['is_all_day']),
      reminderMode: enumValue(
        ReminderPreference.values,
        json['reminder_mode'] as String? ?? 'inheritDefault',
      ),
      reminderMinutesBefore: json['reminder_minutes_before'] as int?,
      createdAt: json['created_at'] as String,
      updatedAt: json['updated_at'] as String,
      completedAt: json['completed_at'] as String?,
      deletedAt: json['deleted_at'] as String?,
      version: json['version'] as int? ?? 1,
    );
  }

  Map<String, Object?> toJson() => {
        'id': id,
        'title': title,
        'status': status.name,
        'priority': priority.name,
        'original_date': originalDate,
        'original_time': originalTime,
        'original_is_all_day': originalIsAllDay ? 1 : 0,
        'scheduled_date': scheduledDate,
        'scheduled_time': scheduledTime,
        'is_all_day': isAllDay ? 1 : 0,
        'reminder_mode': reminderMode.name,
        'reminder_minutes_before': reminderMinutesBefore,
        'created_at': createdAt,
        'updated_at': updatedAt,
        'completed_at': completedAt,
        'deleted_at': deletedAt,
        'version': version,
      };

  Task copyWith({
    String? title,
    TaskStatus? status,
    TaskPriority? priority,
    String? scheduledDate,
    String? scheduledTime,
    bool clearScheduledTime = false,
    bool? isAllDay,
    String? completedAt,
    bool clearCompletedAt = false,
    String? deletedAt,
  }) {
    return Task(
      id: id,
      title: title ?? this.title,
      status: status ?? this.status,
      priority: priority ?? this.priority,
      originalDate: originalDate,
      originalTime: originalTime,
      originalIsAllDay: originalIsAllDay,
      scheduledDate: scheduledDate ?? this.scheduledDate,
      scheduledTime:
          clearScheduledTime ? null : scheduledTime ?? this.scheduledTime,
      isAllDay: isAllDay ?? this.isAllDay,
      reminderMode: reminderMode,
      reminderMinutesBefore: reminderMinutesBefore,
      createdAt: createdAt,
      updatedAt: updatedAt,
      completedAt: clearCompletedAt ? null : completedAt ?? this.completedAt,
      deletedAt: deletedAt ?? this.deletedAt,
      version: version,
    );
  }
}
