// ========= Enums =========>

use std::fmt;

use crate::{
    files::{create_file, delete_file, read_file, update_file},
    utils::{cls, prompt},
};

pub enum Menu {
    Create,
    Read,
    Update,
    Delete,
}

impl Menu {
    pub fn print_options() {
        let list: [Menu; 4] = [Self::Create, Self::Read, Self::Update, Self::Delete];

        println!("Choose an option:\n");

        for (i, opt) in list.iter().enumerate() {
            println!("{}) {opt}", i + 1)
        }

        println!("\nq) Exit.\n");
    }

    pub fn exec_user_option(user_input: u8) {
        cls();

        let choice: Option<Menu> = Menu::from_input(user_input);

        match choice {
            Some(Menu::Create) => create_file(&prompt("file?")),
            Some(Menu::Read) => read_file(&prompt("file?")),
            Some(Menu::Update) => update_file(&prompt("file?")),
            Some(Menu::Delete) => delete_file(&prompt("file?")),
            None => println!("Next time, choose a NUMBER option from the menu :)"),
        }
    }

    fn from_input(input: u8) -> Option<Self> {
        match input {
            b'1' => Some(Self::Create),
            b'2' => Some(Self::Read),
            b'3' => Some(Self::Update),
            b'4' => Some(Self::Delete),
            _ => None,
        }
    }
}

impl fmt::Display for Menu {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Create => "Create a file.",
            Self::Read => "Read a file.",
            Self::Update => "Update a file.",
            Self::Delete => "Delete a file.",
        };

        f.pad(s)
    }
}
