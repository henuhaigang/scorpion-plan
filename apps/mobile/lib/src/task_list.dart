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
            '允许通知后，DayFlow 会在定时任务开始前 10 分钟提醒你。'
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
      return const Center(child: CircularProgressIndicator());
    }
    if (_error != null) {
      return Center(
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            const Text('无法加载任务'),
            Padding(
              padding: const EdgeInsets.all(16),
              child: Text(_error!, textAlign: TextAlign.center),
            ),
            FilledButton(onPressed: _load, child: const Text('重试')),
          ],
        ),
      );
    }
    if (widget.showHistory) {
      final items = _history ?? const <Task>[];
      if (items.isEmpty) return const Center(child: Text('暂无历史记录'));
      return _taskList(items);
    }

    final board = _board;
    if (board == null) return const SizedBox.shrink();
    if (board.todayTimed.isEmpty &&
        board.todayUntimed.isEmpty &&
        board.legacy.isEmpty &&
        board.future.isEmpty) {
      return const Center(
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            Icon(Icons.task_alt, size: 64, color: Colors.grey),
            SizedBox(height: 16),
            Text('今天还没有任务'),
            SizedBox(height: 8),
            Text('点击右上角 + 添加任务'),
          ],
        ),
      );
    }

    return RefreshIndicator(
      onRefresh: _load,
      child: ListView(
        padding: const EdgeInsets.only(top: 4, bottom: 24),
        children: [
          _section('今日', board.todayTimed, board: board),
          _section('待安排', board.todayUntimed, board: board),
          if (board.legacy.isNotEmpty)
            ExpansionTile(
              title: Text('之前未完成 (${board.legacy.length})'),
              children: [_taskList(board.legacy, board: board)],
            ),
          _section('未来', board.future, board: board),
        ],
      ),
    );
  }

  Widget _section(String title, List<Task> tasks, {TaskBoard? board}) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Padding(
          padding: const EdgeInsets.fromLTRB(20, 18, 20, 10),
          child: Row(
            children: [
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
                  color: Theme.of(context).colorScheme.primaryContainer,
                  borderRadius: BorderRadius.circular(20),
                ),
                child: Text(
                  '${tasks.length}',
                  style: Theme.of(context).textTheme.labelSmall?.copyWith(
                        color: Theme.of(context).colorScheme.onPrimaryContainer,
                        fontWeight: FontWeight.w700,
                      ),
                ),
              ),
            ],
          ),
        ),
        if (tasks.isEmpty)
          Padding(
            padding: const EdgeInsets.fromLTRB(20, 0, 20, 4),
            child: Text(
              '暂无任务',
              style: Theme.of(context).textTheme.bodySmall?.copyWith(
                    color: Theme.of(context).colorScheme.onSurfaceVariant,
                  ),
            ),
          )
        else
          _taskList(tasks, board: board),
      ],
    );
  }

  Widget _taskList(List<Task> tasks, {TaskBoard? board}) {
    return Column(
      children: tasks
          .map(
            (task) => Padding(
              padding: const EdgeInsets.fromLTRB(16, 0, 16, 8),
              child: Card(
                child: _TaskTile(
                  task: task,
                  overdue: board?.isOverdue(task) ?? false,
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
  final bool busy;
  final VoidCallback onComplete;
  final VoidCallback onEdit;
  final VoidCallback onCancel;
  final VoidCallback onRestore;
  final VoidCallback onDelete;

  const _TaskTile({
    required this.task,
    required this.overdue,
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
    return ListTile(
      visualDensity: const VisualDensity(vertical: -1),
      contentPadding: const EdgeInsets.symmetric(horizontal: 12, vertical: 1),
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
        ),
      ),
      subtitle: Padding(
        padding: const EdgeInsets.only(top: 3),
        child: Text(
          '${task.scheduledDate}${task.scheduledTime == null ? '' : ' ${task.scheduledTime}'}'
          '${overdue ? ' · 已逾期' : ''}',
          style: Theme.of(context).textTheme.bodySmall?.copyWith(
                color: overdue
                    ? Theme.of(context).colorScheme.error
                    : Theme.of(context).colorScheme.onSurfaceVariant,
              ),
        ),
      ),
      trailing: PopupMenuButton<String>(
        enabled: !busy,
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
