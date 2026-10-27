import 'package:sqflite/sqflite.dart';
import 'package:path/path.dart' as p;
import '../domain/task.dart';

/// SQLite repository for tasks.
class TaskRepository {
  Database? _db;

  Future<Database> _getDatabase() async {
    if (_db != null) return _db!;
    final dbPath = await getDatabasesPath();
    final path = p.join(dbPath, 'dayflow.sqlite');
    _db = await openDatabase(
      path,
      version: 1,
      onCreate: (db, version) async {
        await db.execute('''
          CREATE TABLE IF NOT EXISTS tasks (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            status TEXT NOT NULL DEFAULT 'todo',
            priority TEXT NOT NULL DEFAULT 'normal',
            scheduled_date TEXT,
            scheduled_time TEXT,
            is_all_day INTEGER NOT NULL DEFAULT 0,
            reminder_mode TEXT NOT NULL DEFAULT 'inheritDefault',
            reminder_minutes_before INTEGER,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            version INTEGER NOT NULL DEFAULT 1
          )
        ''');
        await db.execute('''
          CREATE TABLE IF NOT EXISTS app_settings (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
          )
        ''');
      },
    );
    return _db!;
  }

  Future<List<Task>> getAllTasks() async {
    final db = await _getDatabase();
    final rows = await db.query('tasks', orderBy: 'scheduled_date ASC');
    return rows.map((row) => Task.fromJson(row)).toList();
  }

  Future<Task?> getTask(String id) async {
    final db = await _getDatabase();
    final rows = await db.query('tasks', where: 'id = ?', whereArgs: [id]);
    if (rows.isEmpty) return null;
    return Task.fromJson(rows.first);
  }

  Future<void> insertTask(Task task) async {
    final db = await _getDatabase();
    await db.insert('tasks', task.toJson(), conflictAlgorithm: ConflictAlgorithm.replace);
  }

  Future<void> updateTask(Task task) async {
    final db = await _getDatabase();
    await db.update(
      'tasks',
      task.toJson(),
      where: 'id = ? AND version = ?',
      whereArgs: [task.id, task.version - 1],
    );
  }

  Future<void> deleteTask(String id) async {
    final db = await _getDatabase();
    await db.delete('tasks', where: 'id = ?', whereArgs: [id]);
  }

  Future<void> close() async {
    await _db?.close();
    _db = null;
  }
}
