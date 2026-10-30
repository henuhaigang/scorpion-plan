import 'dart:async';
import 'dart:io';

import 'package:flutter/material.dart';

import 'application/task_service.dart';
import 'platform/mobile_reminder_service.dart';
import 'platform/mobile_task_sync.dart';
import 'platform/task_widget_service.dart';
import 'repo/task_repository.dart';
import 'task_list.dart';

class HomePage extends StatefulWidget {
  final TaskService? service;
  final MobileTaskSync? platformSync;

  const HomePage({super.key, this.service, this.platformSync});

  @override
  State<HomePage> createState() => _HomePageState();
}

class _HomePageState extends State<HomePage> with WidgetsBindingObserver {
  final GlobalKey<TaskListState> _taskListKey = GlobalKey<TaskListState>();
  late final TaskService _service;
  MobileReminderService? _reminders;
  MobileTaskSync? _platformSync;
  bool _showHistory = false;
  String? _platformIssue;

  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addObserver(this);
    if (widget.service == null) {
      final repository = TaskRepository();
      _service = TaskService(repository);
      _reminders = MobileReminderService();
      _platformSync = widget.platformSync ??
          MobileTaskSync(
            tasks: repository,
            reminders: _reminders!,
            widgets: TaskWidgetService(),
          );
      WidgetsBinding.instance.addPostFrameCallback((_) {
        unawaited(_synchronize().then((_) async {
          await _taskListKey.currentState?.refresh();
        }));
      });
    } else {
      _service = widget.service!;
      _platformSync = widget.platformSync;
    }
  }

  @override
  void dispose() {
    WidgetsBinding.instance.removeObserver(this);
    super.dispose();
  }

  @override
  void didChangeAppLifecycleState(AppLifecycleState state) {
    if (state == AppLifecycleState.resumed) {
      unawaited(_synchronize().then((_) async {
        await _taskListKey.currentState?.refresh();
      }));
    }
  }

  Future<String?> _synchronize({
    bool requestNotificationPermission = false,
  }) async {
    final issue = await _platformSync?.synchronize(
      requestNotificationPermission: requestNotificationPermission,
    );
    if (mounted && issue != _platformIssue) {
      setState(() => _platformIssue = issue);
    }
    return issue;
  }

  Future<bool> _hasNotificationPermission() async =>
      await _reminders?.hasPermission() ?? true;

  Future<void> _openReminderPermissionDialog() async {
    final shouldRequest = await showDialog<bool>(
      context: context,
      builder: (context) => AlertDialog(
        title: const Text('开启任务提醒'),
        content: const Text(
          '允许通知后，DayFlow 会在定时任务开始前 10 分钟提醒你。'
          '系统可能会延迟通知，提醒不会影响任务保存。',
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
    if (shouldRequest == true) {
      await _synchronize(requestNotificationPermission: true);
    }
  }

  Future<void> _openWidgetInstructions() async {
    final instructions = Platform.isIOS
        ? '在主屏幕长按空白处，点“编辑”>“添加小组件”，搜索 DayFlow，选择尺寸后添加。'
        : Platform.isAndroid
            ? '在主屏幕长按空白处，打开“小组件”，找到 DayFlow 后将小组件拖到桌面。'
            : '请在设备主屏幕的小组件列表中找到 DayFlow 并添加。';

    await showDialog<void>(
      context: context,
      builder: (context) => AlertDialog(
        title: const Text('添加 DayFlow 小组件'),
        content: Text(
          '$instructions\n\n小组件由系统管理，应用不能自动放到主屏幕。'
          '打开 DayFlow 后，今日任务和之前未完成的任务会同步到小组件。',
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(context),
            child: const Text('知道了'),
          ),
          FilledButton(
            onPressed: () {
              Navigator.pop(context);
              unawaited(_refreshWidgetSnapshot());
            },
            child: const Text('同步小组件数据'),
          ),
        ],
      ),
    );
  }

  Future<void> _refreshWidgetSnapshot() async {
    final issue = await _synchronize();
    if (!mounted || issue != null) return;
    ScaffoldMessenger.of(context).showSnackBar(
      const SnackBar(content: Text('小组件数据已更新')),
    );
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('DayFlow'),
        actions: [
          IconButton(
            icon: Icon(_showHistory ? Icons.today : Icons.history),
            onPressed: () => setState(() => _showHistory = !_showHistory),
            tooltip: _showHistory ? '任务' : '历史',
          ),
          IconButton(
            icon: const Icon(Icons.add),
            onPressed: () => _taskListKey.currentState?.showQuickAdd(),
            tooltip: '添加任务',
          ),
          if (_reminders != null)
            IconButton(
              icon: const Icon(Icons.notifications_active_outlined),
              onPressed: _openReminderPermissionDialog,
              tooltip: '开启任务提醒',
            ),
          if (_platformSync != null)
            IconButton(
              icon: const Icon(Icons.widgets_outlined),
              onPressed: _openWidgetInstructions,
              tooltip: '添加桌面小组件',
            ),
        ],
      ),
      body: Column(
        children: [
          if (_platformIssue != null)
            MaterialBanner(
              content: Text(_platformIssue!),
              actions: [
                TextButton(
                  onPressed: _openReminderPermissionDialog,
                  child: const Text('提醒设置'),
                ),
                TextButton(
                  onPressed: () => setState(() => _platformIssue = null),
                  child: const Text('关闭'),
                ),
              ],
            ),
          Expanded(
            child: TaskList(
              key: _taskListKey,
              service: _service,
              showHistory: _showHistory,
              hasNotificationPermission: _hasNotificationPermission,
              onTasksChanged: _synchronize,
            ),
          ),
        ],
      ),
    );
  }
}
