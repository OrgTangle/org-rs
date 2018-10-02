  use std::path::Path;
  use std::path::PathBuf;

  use std::str::Lines;

  use std::fs;
  use std::io;
  use std::env;
  #[derive(Debug)]
  struct TangleError {
      report: String,
  }

  impl TangleError {
      fn new (report: &str) -> Self {
          TangleError {
              report: report.to_string (),
          }
      }
  }
  const BLOCK_BEGIN: &'static str = "#+begin_src ";
  const BLOCK_END: &'static str = "#+end_src";

  fn block_begin_line_p (line: &str) -> bool {
      line .trim_start () .starts_with (BLOCK_BEGIN)
  }

  fn block_end_line_p (line: &str) -> bool {
      line .trim_start () .starts_with (BLOCK_END)
  }

  const DESTINATION_PREFIX: &'static str = "#+property: tangle ";

  fn destination_line_p (line: &str) -> bool {
      line .trim_start () .starts_with (DESTINATION_PREFIX)
  }
  fn find_destination (string: &str) -> Option <String> {
      for line in string.lines () {
          if destination_line_p (line) {
              let destination = &line [DESTINATION_PREFIX.len () ..];
              let destination = destination.trim_end ();
              return Some (destination.to_string ());
          }
      }
      None
  }

  #[test]
  fn test_find_destination () {
      let example = "#+property: tangle core.rs";
      let destination = find_destination (example) .unwrap ();
      assert_eq! (destination, "core.rs");
  }
    fn tangle_collect (
        result: &mut String,
        lines: &mut Lines,
    ) -> Result <(), TangleError> {
        for line in lines {
            if block_end_line_p (line) {
                return Ok (());
            } else {
                result.push_str (line);
                result.push ('\n');
            }
        }
        let error = TangleError::new ("block_end mismatch");
        Err (error)
    }
    fn tangle (string: &str) -> Result <String, TangleError> {
        let mut result = String::new ();
        let mut lines = string.lines ();
        while let Some (line) = lines.next () {
            if block_begin_line_p (line) {
                tangle_collect (&mut result, &mut lines)?;
            }
        }
        Ok (result)
    }
    #[test]
    fn test_tangle () {
        let example = format! (
            "{}\n{}\n{}\n{}\n",
            "#+begin_src rust",
            "hi",
            "hi",
            "#+end_src",
        );
        let expect = format! (
            "{}\n{}\n",
            "hi",
            "hi",
        );
        let result = tangle (&example) .unwrap ();
        assert_eq! (expect, result);
        let example = format! (
            "{}\n{}\n{}\n{}\n",
            "    #+begin_src rust",
            "    hi",
            "    hi",
            "    #+end_src",
        );
        let expect = format! (
            "{}\n{}\n",
            "    hi",
            "    hi",
        );
        let result = tangle (&example) .unwrap ();
        assert_eq! (expect, result);
    }
    fn good_path_p (path: &Path) -> bool {
        for component in path.iter () {
            if let Some (string) = component.to_str () {
                if string.starts_with ('.') {
                    if ! string .chars () .all (|x| x == '.') {
                        return false;
                    }
                }
            } else {
                return false;
            }
        }
        true
    }
    pub fn org_file_p (file: &Path) -> bool {
        if let Some (os_string) = file.extension () {
            if let Some (string) = os_string.to_str () {
                string == "org"
            } else {
                false
            }
        } else {
            false
        }
    }
    pub fn file_tangle (file: &Path) -> io::Result <()> {
        if ! org_file_p (file) {
            return Ok (());
        }
        println! ("- tangle : {:?}", file);
        let string = fs::read_to_string (file)?;
        if let Some (destination) = find_destination (&string) {
            let result = tangle (&string) .unwrap ();
            let mut destination_path = PathBuf::new ();
            destination_path.push (file);
            destination_path.pop ();
            destination_path.push (destination);
            fs::write (&destination_path, result)
        } else {
            Ok (())
        }
    }
    pub fn dir_tangle (dir: &Path) -> io::Result <()> {
        for entry in dir.read_dir ()? {
            if let Ok (entry) = entry {
                if good_path_p (&entry.path ()) {
                    if entry.file_type ()? .is_file () {
                        file_tangle (&entry.path ())?
                    }
                }
            }
        }
        Ok (())
    }
    pub fn dir_tangle_rec (dir: &Path) -> io::Result <()> {
        for entry in dir.read_dir ()? {
            if let Ok (entry) = entry {
                if good_path_p (&entry.path ()) {
                    if entry.file_type ()? .is_file () {
                        file_tangle (&entry.path ())?
                    } else if entry.file_type ()? .is_dir () {
                        dir_tangle_rec (&entry.path ())?
                    }
                }
            }
        }
        Ok (())
    }
    pub fn absolute_lize (path: &Path) -> PathBuf {
        if path.is_relative () {
            let mut absolute_path = env::current_dir () .unwrap ();
            absolute_path.push (path);
            absolute_path
        } else {
            path.to_path_buf ()
        }
    }
