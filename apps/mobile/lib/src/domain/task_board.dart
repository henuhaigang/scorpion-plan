import 'task.dart';

class TaskBoard {
  final DateTime observedAt;
  final List<Task> todayTimed;
  final List<Task> todayUntimed;
  final List<Task> legacy;
  final List<Task> future;

  const TaskBoard({
    required this.observedAt,
    required this.todayTimed,
    required this.todayUntimed,
    required this.legacy,
    required this.future,
  });

  factory TaskBoard.fromTasks(List<Task> tasks, DateTime now) {
    final today = DateTime(now.year, now.month, now.day);
    final timed = <Task>[];
    final untimed = <Task>[];
    final legacy = <Task>[];
    final future = <Task>[];

    for (final task in tasks.where((task) => task.status == TaskStatus.todo)) {
      final date = DateTime.parse(task.scheduledDate);
      final day = DateTime(date.year, date.month, date.day);
      if (day.isBefore(today)) {
        legacy.add(task);
      } else if (day.isAfter(today)) {
        future.add(task);
      } else if (task.scheduledTime == null) {
        untimed.add(task);
      } else {
        timed.add(task);
      }
    }

    timed.sort((a, b) {
      final timeOrder = a.scheduledTime!.compareTo(b.scheduledTime!);
      return timeOrder != 0 ? timeOrder : a.id.compareTo(b.id);
    });
    untimed.sort((a, b) => a.createdAt.compareTo(b.createdAt));
    legacy.sort((a, b) => a.scheduledDate.compareTo(b.scheduledDate));
    future.sort((a, b) {
      final dateOrder = a.scheduledDate.compareTo(b.scheduledDate);
      if (dateOrder != 0) return dateOrder;
      return (a.scheduledTime ?? '').compareTo(b.scheduledTime ?? '');
    });

    return TaskBoard(
      observedAt: now,
      todayTimed: timed,
      todayUntimed: untimed,
      legacy: legacy,
      future: future,
    );
  }

  bool isOverdue(Task task) {
    final now = observedAt;
    if (task.scheduledDate != _dateString(now) || task.scheduledTime == null) {
      return false;
    }
    final parts = task.scheduledTime!.split(':').map(int.parse).toList();
    final scheduled =
        DateTime(now.year, now.month, now.day, parts[0], parts[1]);
    return scheduled.isBefore(now);
  }

  static String _dateString(DateTime date) =>
      '${date.year.toString().padLeft(4, '0')}-'
      '${date.month.toString().padLeft(2, '0')}-'
      '${date.day.toString().padLeft(2, '0')}';
}
