use std::{ffi::OsString, path::PathBuf};

use windows::{
    core::PCWSTR,
    Win32::{
        Foundation::HMODULE,
        System::{Environment::GetCommandLineW, LibraryLoader::GetModuleFileNameW},
    },
};

const MAX_FILENAME_SIZE: usize = 512;

// fn compute_program_length(const wchar_t* commandline) -> usize
// {
//   int i = 0;

//   if (commandline[0] == L'"') {
//     // Wait till end of string
//     i++;

//     for (;;) {
//       wchar_t c = commandline[i++];

//       if (c == 0)
//         return i - 1;
//       else if (c == L'\\')
//         i++;
//       else if (c == L'"')
//         return i;
//     }
//   } else {
//     for (;;) {
//       wchar_t c = commandline[i++];

//       if (c == 0)
//         return i - 1;
//       else if (c == L'\\')
//         i++;
//       else if (c == L' ')
//         return i;
//     }
//   }
// }

fn main() {
    let command_line: PCWSTR = unsafe { GetCommandLineW() };
    let file_name = std::env::current_exe().unwrap();

    println!("Hello, world!");
}
