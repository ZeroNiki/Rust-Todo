use figlet_rs::FIGfont;
use std::io::{self, BufRead};

use rusqlite::{params, Connection, Result};

#[derive(Debug)]
struct TodoItem {
    title: String,
}

fn choice_page() -> i32 {
    println!("\n1.Add todo\n2.Display todos\n3.Change status\n4.Exit");

    let stdin = io::stdin();
    let nav = stdin
        .lock()
        .lines()
        .next()
        .unwrap()
        .unwrap()
        .parse::<i32>()
        .unwrap();

    nav
}

fn delete_from_db(delete_id: i32) -> Result<()> {
    let conn = Connection::open("todo_list.db")?;

    conn.execute("DELETE FROM todos WHERE id = ?", params![delete_id])?;

    Ok(())
}

fn reset_autoincrement(conn: &Connection) -> Result<()> {
    conn.execute("BEGIN", [])?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS todos_temp AS SELECT * FROM todos",
        [],
    )?;
    conn.execute("DROP TABLE IF EXISTS todos", [])?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS todos (
                  id INTEGER PRIMARY KEY AUTOINCREMENT,
                  title TEXT NOT NULL
                  )",
        [],
    )?;
    conn.execute("INSERT INTO todos SELECT * FROM todos_temp", [])?;
    conn.execute("DROP TABLE IF EXISTS todos_temp", [])?;
    conn.execute("COMMIT", [])?;
    Ok(())
}

fn main() -> Result<()> {
    let standard_font = FIGfont::standard().unwrap();
    let figure = standard_font.convert("TODO");
    println! {"\n{}\n", figure.unwrap()};

    let conn = Connection::open("todo_list.db")?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS todos (
                  id INTEGER PRIMARY KEY AUTOINCREMENT,
                  title TEXT NOT NULL
                  )",
        [],
    )?;

    while true {
        let input = choice_page();

        match input {
            1 => {
                println!("Todo title:");
                let title = io::stdin().lock().lines().next().unwrap().unwrap();
                println!("\n");

                let todo_data = TodoItem { title: title };

                conn.execute("insert into todos (title) values(?)", [todo_data.title])?;
            }

            2 => {
                let mut data = conn.prepare("select * from todos").unwrap();

                let todo_iter = data
                    .query_map([], |row| {
                        Ok((row.get::<_, i32>(0)?, row.get::<_, String>(1)?))
                    })
                    .unwrap();

                for todo in todo_iter {
                    if let Ok((id, title)) = todo {
                        println!("{} {}", id, title);
                    }
                }
            }

            3 => {
                println!("Todo id to delete:");
                let delete_id = io::stdin()
                    .lock()
                    .lines()
                    .next()
                    .unwrap()
                    .unwrap()
                    .parse::<i32>()
                    .unwrap();

                if let Err(err) = delete_from_db(delete_id) {
                    println!("Error! Cannot delete item: {:?}", err);
                } else {
                    println!(
                        "Success! {} has been deleted from table",
                        delete_id.to_string()
                    );

                    println!("Resetting ID...");
                    reset_autoincrement(&conn)?;
                    println!("ID reset successfully!")
                }
            }

            4 => {
                println!("Goodbye!");
                break;
            }

            _ => println!("Error!"),
        }
    }
    Ok(())
}
