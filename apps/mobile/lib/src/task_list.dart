import 'package:flutter/material.dart';
import 'application/task_service.dart';
import 'domain/task.dart';
import 'domain/task_board.dart';

class TaskList extends StatefulWidget {
  final TaskService service;
  final bool showHistory;
  final Future<bool> Function()? hasNotificationPermission;
  final Future<String?> Function({bool requestNotificationPermission})?
      onTasksChanged;

  const TaskList({
    super.key,
    required this.service,
    required this.showHistory,
    this.hasNotificationPermission,
    this.onTasksChanged,
  });

  @override
  State<TaskList> createState() => TaskListState();
}

class TaskListState extends State<TaskList> {
  TaskBoard? _board;
  List<Task>? _history;
  String? _error;
  bool _loading = true;
  bool _busy = false;

  @override
  void initState() {
    super.initState();
    _load();
  }

  @override
  void didUpdateWidget(covariant TaskList oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (oldWidget.showHistory != widget.showHistory) _load();
  }

  Future<void> _load() async {
    setState(() {
      _loading = true;
      _error = null;
    });
    try {
      if (widget.showHistory) {
        _history = await widget.service.loadHistory();
      } else {
        _board = await widget.service.loadBoard();
      }
    } catch (error) {
      _error = error.toString();
    } finally {
      if (mounted) setState(() => _loading = false);
    }
  }

  Future<void> showQuickAdd() => _showTaskEditor();

  Future<void> refresh() => _load();

  Future<void> _showTaskEditor([Task? task]) async {
    final draft = await showDialog<TaskDraft>(
      context: context,
      builder: (context) => TaskEditorDialog(
        task: task,
        initialDate: widget.service.todayDate(),
      ),
    );
    if (draft == null) return;
    var requestNotificationPermission = false;
    if (draft.time != null &&
        widget.hasNotificationPermission != null &&
        !await widget.hasNotificationPermission!()) {
      requestNotificationPermission =
          await _confirmNotificationPermission() ?? false;
    }
    await _run(() async {
      if (task == null) {
        await widget.service.createTask(
          title: draft.title,
          date: draft.date,
          time: draft.time,
          priority: draft.priority,
        );
      } else {
        await widget.service.updateTask(
          task,
          title: draft.title,
          date: draft.date,
          time: draft.time,
          priority: draft.priority,
        );
      }
    }, requestNotificationPermission: requestNotificationPermission);
  }

  Future<bool?> _confirmNotificationPermission() => showDialog<bool>(
        context: context,
        builder: (context) => AlertDialog(
          title: const Text('开启任务提醒'),
          content: const Text(
            '允许通知后，Splan 会在定时任务开始前 10 分钟提醒你。'
            '任务仍会正常保存；系统可能会延迟通知。',
          ),
          actions: [
            TextButton(
              onPressed: () => Navigator.pop(context, false),
              child: const Text('暂不开启'),
            ),
            FilledButton(
              onPressed: () => Navigator.pop(context, true),
              child: const Text('继续'),
            ),
          ],
        ),
      );

  Future<void> _run(
    Future<void> Function() action, {
    bool requestNotificationPermission = false,
  }) async {
    setState(() => _busy = true);
    try {
      await action();
      await _load();
      final issue = await widget.onTasksChanged?.call(
        requestNotificationPermission: requestNotificationPermission,
      );
      if (issue != null && mounted) {
        ScaffoldMessenger.of(context)
            .showSnackBar(SnackBar(content: Text(issue)));
      }
    } catch (error) {
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(
              content: Text(error.toString().replaceFirst('Bad state: ', ''))),
        );
      }
    } finally {
      if (mounted) setState(() => _busy = false);
    }
  }

  Future<void> _complete(Task task) => _run(() async {
        if (task.status == TaskStatus.completed) {
          await widget.service.undoCompletion(task);
        } else {
          await widget.service.completeTask(task);
        }
      });

  Future<void> _restore(Task task) => _run(() async {
        await widget.service.restoreCanceledTask(task);
      });

  Future<void> _cancel(Task task) => _run(() async {
        await widget.service.cancelTask(task);
      });

  Future<void> _delete(Task task) => _run(() async {
        await widget.service.softDeleteTask(task);
      });

  @override
  Widget build(BuildContext context) {
    if (_loading) {
      return Center(
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            const CircularProgressIndicator.adaptive(),
            const SizedBox(height: 14),
            Text(
              '正在整理你的任务…',
              style: Theme.of(context).textTheme.bodyMedium?.copyWith(
                    color: Theme.of(context).colorScheme.onSurfaceVariant,
                  ),
            ),
          ],
        ),
      );
    }
    if (_error != null) {
      return Center(
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            Icon(
              Icons.error_outline_rounded,
              size: 36,
              color: Theme.of(context).colorScheme.error,
            ),
            const SizedBox(height: 12),
            Text(
              '无法加载任务',
              style: Theme.of(context).textTheme.titleMedium,
            ),
            Padding(
              padding: const EdgeInsets.all(20),
              child: Text(
                _error!,
                textAlign: TextAlign.center,
                style: Theme.of(context).textTheme.bodySmall?.copyWith(
                      color: Theme.of(context).colorScheme.onSurfaceVariant,
                    ),
              ),
            ),
            FilledButton(onPressed: _load, child: const Text('重试')),
          ],
        ),
      );
    }
    if (widget.showHistory) {
      final items = _history ?? const <Task>[];
      if (items.isEmpty) {
        return Center(
          child: _emptyState(
            Icons.history_rounded,
            '暂无历史记录',
            '完成或取消的任务会显示在这里',
          ),
        );
      }
      return ListView(
        padding: const EdgeInsets.fromLTRB(16, 8, 16, 24),
        children: [
          _section('历史记录', Icons.history_rounded, items),
        ],
      );
    }

    final board = _board;
    if (board == null) return const SizedBox.shrink();
    return RefreshIndicator(
      onRefresh: _load,
      child: ListView(
        physics: const AlwaysScrollableScrollPhysics(),
        padding: const EdgeInsets.fromLTRB(16, 8, 16, 28),
        children: [
          _todayOverview(board),
          if (board.todayTimed.isEmpty &&
              board.todayUntimed.isEmpty &&
              board.legacy.isEmpty &&
              board.future.isEmpty)
            _emptyState(
              Icons.wb_sunny_outlined,
              '今天还没有安排',
              '先记下一件最重要的事，其他的慢慢来',
            ),
          _section(
            '定时任务',
            Icons.schedule_rounded,
            board.todayTimed,
            board: board,
          ),
          _section(
            '待安排',
            Icons.event_note_rounded,
            board.todayUntimed,
            board: board,
          ),
          if (board.legacy.isNotEmpty)
            Card(
              clipBehavior: Clip.antiAlias,
              child: ExpansionTile(
                leading: Icon(
                  Icons.history_rounded,
                  color: Theme.of(context).colorScheme.onSurfaceVariant,
                ),
                title: const Text('之前未完成'),
                subtitle: Text('${board.legacy.length} 项任务'),
                children: [_taskList(board.legacy, board: board)],
              ),
            ),
          _section(
            '接下来',
            Icons.upcoming_outlined,
            board.future,
            board: board,
          ),
        ],
      ),
    );
  }

  Widget _todayOverview(TaskBoard board) {
    final date = board.observedAt;
    const weekdays = ['一', '二', '三', '四', '五', '六', '日'];
    final count = board.todayTimed.length + board.todayUntimed.length;
    final colors = Theme.of(context).colorScheme;

    return Container(
      margin: const EdgeInsets.only(bottom: 12),
      padding: const EdgeInsets.all(22),
      decoration: BoxDecoration(
        gradient: LinearGradient(
          begin: Alignment.topLeft,
          end: Alignment.bottomRight,
          colors: [
            colors.primary,
            Color.lerp(colors.primary, colors.tertiary, 0.28)!,
          ],
        ),
        borderRadius: BorderRadius.circular(24),
      ),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  '星期${weekdays[date.weekday - 1]} · ${date.month}月${date.day}日',
                  style: Theme.of(context).textTheme.labelLarge?.copyWith(
                        color: colors.onPrimary.withValues(alpha: 0.82),
                        fontWeight: FontWeight.w500,
                      ),
                ),
                const SizedBox(height: 14),
                Text(
                  '今天，按自己的节奏来',
                  style: Theme.of(context).textTheme.headlineSmall?.copyWith(
                        color: colors.onPrimary,
                        fontWeight: FontWeight.w700,
                        letterSpacing: -0.5,
                      ),
                ),
                const SizedBox(height: 8),
                Text(
                  count == 0 ? '今天留白，正好喘口气' : '今天有 $count 项安排',
                  style: Theme.of(context).textTheme.bodyMedium?.copyWith(
                        color: colors.onPrimary.withValues(alpha: 0.84),
                      ),
                ),
              ],
            ),
          ),
          const SizedBox(width: 8),
          Container(
            width: 42,
            height: 42,
            decoration: BoxDecoration(
              color: colors.onPrimary.withValues(alpha: 0.14),
              borderRadius: BorderRadius.circular(15),
            ),
            child: Icon(
              Icons.wb_sunny_rounded,
              color: colors.onPrimary,
              size: 22,
            ),
          ),
        ],
      ),
    );
  }

  Widget _emptyState(IconData icon, String title, String subtitle) {
    final colors = Theme.of(context).colorScheme;
    return Card(
      child: Padding(
        padding: const EdgeInsets.symmetric(horizontal: 20, vertical: 28),
        child: Column(
          children: [
            Container(
              width: 54,
              height: 54,
              decoration: BoxDecoration(
                color: colors.primaryContainer,
                borderRadius: BorderRadius.circular(19),
              ),
              child: Icon(icon, color: colors.primary, size: 27),
            ),
            const SizedBox(height: 14),
            Text(
              title,
              style: Theme.of(context).textTheme.titleMedium?.copyWith(
                    fontWeight: FontWeight.w700,
                  ),
            ),
            const SizedBox(height: 6),
            Text(
              subtitle,
              textAlign: TextAlign.center,
              style: Theme.of(context).textTheme.bodySmall?.copyWith(
                    color: colors.onSurfaceVariant,
                  ),
            ),
          ],
        ),
      ),
    );
  }

  Widget _section(
    String title,
    IconData icon,
    List<Task> tasks, {
    TaskBoard? board,
  }) {
    if (tasks.isEmpty) return const SizedBox.shrink();

    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Padding(
          padding: const EdgeInsets.fromLTRB(4, 16, 4, 10),
          child: Row(
            children: [
              Icon(
                icon,
                size: 19,
                color: Theme.of(context).colorScheme.primary,
              ),
              const SizedBox(width: 8),
              Text(
                title,
                style: Theme.of(context).textTheme.titleMedium?.copyWith(
                      fontWeight: FontWeight.w700,
                      letterSpacing: -0.2,
                    ),
              ),
              const SizedBox(width: 8),
              Container(
                padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 3),
                decoration: BoxDecoration(
                  color: Theme.of(context).colorScheme.surfaceContainerHighest,
                  borderRadius: BorderRadius.circular(20),
                ),
                child: Text(
                  '${tasks.length}',
                  style: Theme.of(context).textTheme.labelSmall?.copyWith(
                        color: Theme.of(context).colorScheme.onSurfaceVariant,
                        fontWeight: FontWeight.w700,
                      ),
                ),
              ),
            ],
          ),
        ),
        _taskList(tasks, board: board),
      ],
    );
  }

  Widget _taskList(List<Task> tasks, {TaskBoard? board}) {
    final today = board?.observedAt;
    final todayDate = today == null
        ? null
        : '${today.year.toString().padLeft(4, '0')}-'
            '${today.month.toString().padLeft(2, '0')}-'
            '${today.day.toString().padLeft(2, '0')}';

    return Column(
      children: tasks
          .map(
            (task) => Padding(
              padding: const EdgeInsets.only(bottom: 8),
              child: Card(
                clipBehavior: Clip.antiAlias,
                child: _TaskTile(
                  task: task,
                  overdue: board?.isOverdue(task) ?? false,
                  isToday: task.scheduledDate == todayDate,
                  busy: _busy,
                  onComplete: () => _complete(task),
                  onEdit: () => _showTaskEditor(task),
                  onCancel: () => _cancel(task),
                  onRestore: () => _restore(task),
                  onDelete: () => _delete(task),
                ),
              ),
            ),
          )
          .toList(),
    );
  }
}

class _TaskTile extends StatelessWidget {
  final Task task;
  final bool overdue;
  final bool isToday;
  final bool busy;
  final VoidCallback onComplete;
  final VoidCallback onEdit;
  final VoidCallback onCancel;
  final VoidCallback onRestore;
  final VoidCallback onDelete;

  const _TaskTile({
    required this.task,
    required this.overdue,
    required this.isToday,
    required this.busy,
    required this.onComplete,
    required this.onEdit,
    required this.onCancel,
    required this.onRestore,
    required this.onDelete,
  });

  @override
  Widget build(BuildContext context) {
    final isTodo = task.status == TaskStatus.todo;
    final colors = Theme.of(context).colorScheme;
    final scheduledDate = DateTime.parse(task.scheduledDate);
    final scheduleLabel = isToday
        ? task.scheduledTime ?? '待安排'
        : '${scheduledDate.month}月${scheduledDate.day}日'
            '${task.scheduledTime == null ? '' : ' · ${task.scheduledTime}'}';

    return ListTile(
      minVerticalPadding: 13,
      contentPadding: const EdgeInsets.fromLTRB(12, 4, 8, 4),
      leading: Checkbox(
        value: task.status == TaskStatus.completed,
        shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(6)),
        onChanged: busy || task.status == TaskStatus.canceled
            ? null
            : (_) => onComplete(),
      ),
      title: Text(
        task.title,
        maxLines: 2,
        overflow: TextOverflow.ellipsis,
        style: Theme.of(context).textTheme.bodyMedium?.copyWith(
              fontWeight: FontWeight.w600,
              decoration: isTodo ? null : TextDecoration.lineThrough,
              color: isTodo ? colors.onSurface : colors.onSurfaceVariant,
            ),
      ),
      subtitle: Padding(
        padding: const EdgeInsets.only(top: 8),
        child: Wrap(
          spacing: 6,
          runSpacing: 5,
          children: [
            _TaskPill(
              icon: task.scheduledTime == null
                  ? Icons.event_note_outlined
                  : Icons.schedule_rounded,
              label: scheduleLabel,
              foreground: colors.onSurfaceVariant,
              background: colors.surfaceContainerHighest,
            ),
            if (task.priority == TaskPriority.high)
              _TaskPill(
                icon: Icons.flag_rounded,
                label: '高优先级',
                foreground: colors.tertiary,
                background: colors.tertiaryContainer,
              ),
            if (overdue)
              _TaskPill(
                icon: Icons.error_outline_rounded,
                label: '已逾期',
                foreground: colors.error,
                background: colors.errorContainer,
              ),
          ],
        ),
      ),
      trailing: PopupMenuButton<String>(
        enabled: !busy,
        tooltip: '任务操作',
        icon: const Icon(Icons.more_horiz_rounded),
        onSelected: (value) {
          switch (value) {
            case 'edit':
              onEdit();
              break;
            case 'cancel':
              onCancel();
              break;
            case 'restore':
              onRestore();
              break;
            case 'delete':
              onDelete();
              break;
          }
        },
        itemBuilder: (context) => [
          if (isTodo) ...[
            const PopupMenuItem(value: 'edit', child: Text('编辑')),
            const PopupMenuItem(value: 'cancel', child: Text('取消任务')),
          ],
          if (task.status == TaskStatus.canceled)
            const PopupMenuItem(value: 'restore', child: Text('恢复任务')),
          const PopupMenuItem(value: 'delete', child: Text('删除')),
        ],
      ),
    );
  }
}

class _TaskPill extends StatelessWidget {
  final IconData icon;
  final String label;
  final Color foreground;
  final Color background;

  const _TaskPill({
    required this.icon,
    required this.label,
    required this.foreground,
    required this.background,
  });

  @override
  Widget build(BuildContext context) {
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
      decoration: BoxDecoration(
        color: background,
        borderRadius: BorderRadius.circular(9),
      ),
      child: Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          Icon(icon, size: 13, color: foreground),
          const SizedBox(width: 4),
          Text(
            label,
            style: Theme.of(context).textTheme.labelSmall?.copyWith(
                  color: foreground,
                  fontWeight: FontWeight.w600,
                ),
          ),
        ],
      ),
    );
  }
}

class TaskDraft {
  final String title;
  final String date;
  final String? time;
  final TaskPriority priority;

  const TaskDraft(this.title, this.date, this.time, this.priority);
}

class TaskEditorDialog extends StatefulWidget {
  final Task? task;
  final String initialDate;

  const TaskEditorDialog({super.key, this.task, required this.initialDate});

  @override
  State<TaskEditorDialog> createState() => _TaskEditorDialogState();
}

class _TaskEditorDialogState extends State<TaskEditorDialog> {
  late final TextEditingController _title;
  late DateTime _date;
  TimeOfDay? _time;
  late TaskPriority _priority;

  @override
  void initState() {
    super.initState();
    final task = widget.task;
    _title = TextEditingController(text: task?.title ?? '');
    _date = DateTime.parse(task?.scheduledDate ?? widget.initialDate);
    _time = task?.scheduledTime == null
        ? null
        : TimeOfDay(
            hour: int.parse(task!.scheduledTime!.split(':')[0]),
            minute: int.parse(task.scheduledTime!.split(':')[1]),
          );
    _priority = task?.priority ?? TaskPriority.normal;
  }

  @override
  void dispose() {
    _title.dispose();
    super.dispose();
  }

  String _dateString(DateTime date) =>
      '${date.year.toString().padLeft(4, '0')}-'
      '${date.month.toString().padLeft(2, '0')}-'
      '${date.day.toString().padLeft(2, '0')}';

  @override
  Widget build(BuildContext context) {
    return AlertDialog(
      title: Text(widget.task == null ? '快速添加' : '编辑任务'),
      content: SingleChildScrollView(
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            TextField(
              controller: _title,
              autofocus: true,
              maxLength: 200,
              decoration: const InputDecoration(labelText: '任务标题'),
            ),
            ListTile(
              contentPadding: EdgeInsets.zero,
              title: const Text('日期'),
              subtitle: Text(_dateString(_date)),
              onTap: () async {
                final selected = await showDatePicker(
                  context: context,
                  initialDate: _date,
                  firstDate: DateTime(2000),
                  lastDate: DateTime(2100),
                );
                if (selected != null) setState(() => _date = selected);
              },
            ),
            ListTile(
              contentPadding: EdgeInsets.zero,
              title: const Text('时间'),
              subtitle: Text(_time?.format(context) ?? '不指定时间'),
              trailing: _time == null
                  ? const Icon(Icons.access_time)
                  : IconButton(
                      tooltip: '清除时间',
                      icon: const Icon(Icons.clear),
                      onPressed: () => setState(() => _time = null),
                    ),
              onTap: () async {
                final selected = await showTimePicker(
                  context: context,
                  initialTime: _time ?? const TimeOfDay(hour: 9, minute: 0),
                );
                if (selected != null) setState(() => _time = selected);
              },
            ),
            DropdownButtonFormField<TaskPriority>(
              initialValue: _priority,
              decoration: const InputDecoration(labelText: '优先级'),
              items: const [
                DropdownMenuItem(value: TaskPriority.low, child: Text('低')),
                DropdownMenuItem(value: TaskPriority.normal, child: Text('普通')),
                DropdownMenuItem(value: TaskPriority.high, child: Text('高')),
              ],
              onChanged: (value) {
                if (value != null) setState(() => _priority = value);
              },
            ),
          ],
        ),
      ),
      actions: [
        TextButton(
          onPressed: () => Navigator.pop(context),
          child: const Text('取消'),
        ),
        FilledButton(
          onPressed: () {
            final time = _time == null
                ? null
                : '${_time!.hour.toString().padLeft(2, '0')}:'
                    '${_time!.minute.toString().padLeft(2, '0')}';
            Navigator.pop(
              context,
              TaskDraft(_title.text, _dateString(_date), time, _priority),
            );
          },
          child: Text(widget.task == null ? '添加' : '保存'),
        ),
      ],
    );
  }
}
