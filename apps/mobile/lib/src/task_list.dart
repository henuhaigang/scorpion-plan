import 'package:flutter/material.dart';

class TaskList extends StatelessWidget {
  const TaskList({super.key});

  @override
  Widget build(BuildContext context) {
    // TODO: Load tasks from repository
    return Center(
      child: Column(
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          const Icon(Icons.task_alt, size: 64, color: Colors.grey),
          const SizedBox(height: 16),
          const Text('暂无任务', style: TextStyle(color: Colors.grey)),
          const SizedBox(height: 8),
          TextButton(
            onPressed: () {},
            child: const Text('添加第一个任务'),
          ),
        ],
      ),
    );
  }
}
