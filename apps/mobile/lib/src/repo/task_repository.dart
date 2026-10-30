import 'package:path/path.dart' as p;
import 'package:sqflite/sqflite.dart';

import '../domain/task.dart';

abstract interface class TaskStore {
  Future<List<Task>> getActiveTasks();
  Future<Task?> getTask(String id);
  Future<void> insertTask(Task task);
  Future<void> updateTask(Task task, {required int expectedVersion});
  Future<void> close();
}

class TaskRepository implements TaskStore {
  Database? _db;
  Future<Database>? _openingDatabase;

  Future<Database> _getDatabase() async {
    if (_db != null) return _db!;
    final openingDatabase = _openingDatabase;
    if (openingDatabase != null) return openingDatabase;

    final pendingDatabase = _openDatabase();
    _openingDatabase = pendingDatabase;
    try {
      final database = await pendingDatabase;
      _db = database;
      return database;
    } catch (_) {
      if (identical(_openingDatabase, pendingDatabase)) {
        _openingDatabase = null;
      }
      rethrow;
    }
  }

  Future<Database> _openDatabase() async {
    final dbPath = await getDatabasesPath();
    final path = p.join(dbPath, 'dayflow.sqlite');
    return openDatabase(
      path,
      version: 2,
      onCreate: (db, _) async {
        await db.execute('''
          CREATE TABLE tasks (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            status TEXT NOT NULL CHECK(status IN ('todo', 'completed', 'canceled')),
            priority TEXT NOT NULL CHECK(priority IN ('low', 'normal', 'high')),
            original_date TEXT NOT NULL,
            original_time TEXT,
            original_is_all_day INTEGER NOT NULL DEFAULT 0,
            scheduled_date TEXT NOT NULL,
            scheduled_time TEXT,
            is_all_day INTEGER NOT NULL DEFAULT 0,
            reminder_mode TEXT NOT NULL DEFAULT 'inheritDefault',
            reminder_minutes_before INTEGER,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            completed_at TEXT,
            deleted_at TEXT,
            version INTEGER NOT NULL DEFAULT 1,
            CHECK(is_all_day = 0 OR scheduled_time IS NULL),
            CHECK(original_is_all_day = 0 OR original_time IS NULL)
          )
        ''');
      },
      onUpgrade: (db, oldVersion, _) async {
        if (oldVersion < 2) {
          await db.execute('ALTER TABLE tasks ADD COLUMN original_date TEXT');
          await db.execute('ALTER TABLE tasks ADD COLUMN original_time TEXT');
          await db.execute(
            'ALTER TABLE tasks ADD COLUMN original_is_all_day INTEGER NOT NULL DEFAULT 0',
          );
          await db.execute('ALTER TABLE tasks ADD COLUMN completed_at TEXT');
          await db.execute('ALTER TABLE tasks ADD COLUMN deleted_at TEXT');
          await db.execute('''
            UPDATE tasks
            SET scheduled_date = COALESCE(scheduled_date, date('now', 'localtime')),
                original_date = COALESCE(scheduled_date, date('now', 'localtime')),
                original_time = scheduled_time
          ''');
        }
      },
    );
  }

  @override
  Future<List<Task>> getActiveTasks() async {
    final db = await _getDatabase();
    final rows = await db.query(
      'tasks',
      where: 'deleted_at IS NULL',
      orderBy: 'scheduled_date ASC, scheduled_time ASC, created_at ASC, id ASC',
    );
    return rows.map((row) => Task.fromJson(row)).toList();
  }

  @override
  Future<Task?> getTask(String id) async {
    final db = await _getDatabase();
    final rows = await db.query('tasks', where: 'id = ?', whereArgs: [id]);
    if (rows.isEmpty) return null;
    return Task.fromJson(rows.first);
  }

  @override
  Future<void> insertTask(Task task) async {
    final db = await _getDatabase();
    await db.insert('tasks', task.toJson());
  }

  @override
  Future<void> updateTask(Task task, {required int expectedVersion}) async {
    final db = await _getDatabase();
    final count = await db.update(
      'tasks',
      task.toJson(),
      where: 'id = ? AND version = ? AND deleted_at IS NULL',
      whereArgs: [task.id, expectedVersion],
    );
    if (count != 1) {
      throw StateError('任务已被修改或删除，请刷新后重试');
    }
  }

  @override
  Future<void> close() async {
    final database = _db ?? await _openingDatabase;
    await database?.close();
    _db = null;
    _openingDatabase = null;
  }
}
