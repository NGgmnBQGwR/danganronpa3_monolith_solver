use std::io::{BufWriter, Write};
use std::path::PathBuf;

use crate::errors::MyError;
use crate::map::{MonolithMap, Tile};

const AHK_TEMPLATE: &str = r#"
#SingleInstance Force

makeCircle() {
    Gui New, +E0x20 +AlwaysOnTop +ToolWindow -Caption +HwndHWND
    r := 120
    thickness := 20
    transparency := 240
    color := 0x00CD00

    outer := DllCall("CreateEllipticRgn", "Int", 0, "Int", 0, "Int", r, "Int", r)
    inner := DllCall("CreateEllipticRgn", "Int", thickness, "Int", thickness, "Int", r - thickness, "Int", r - thickness)
    DllCall("CombineRgn", "UInt", outer, "UInt", outer, "UInt", inner, "Int", 3)
    DllCall("SetWindowRgn", "UInt", HWND, "UInt", outer, "UInt", true)

    Gui %HWND%:Color, % color
    Gui %HWND%:Show, X0 Y0 W%r% H%r% NoActivate
    WinSet Transparent, % transparency, % "ahk_id " HWND

    return HWND
}

hCircle := makeCircle()
tiles := ARRAY_MARKER
step := 1

loop {
    if WinActive("ahk_exe Dangan3Win.exe")
    {
        tile := tiles[step]
        x := tile[1] * 80 + 80 - 20
        y := tile[2] * 80 + 80 - 20
        Gui %hCircle%:Show, X%x% Y%y% NoActivate
    }
    else
    {
        Gui %hCircle%:Hide
    }
    sleep 100
}

~q::
    ExitApp
    Return

~LButton::
    if (step < tiles.MaxIndex())
    {
        step += 1
    }
    Return

~RButton::
    if (step > 1)
    {
        step -= 1
    }
    Return
"#;

fn create_array_string(steps: Vec<Tile>) -> String {
    let mut result = String::with_capacity(300);
    result.push_str("[");
    result.push_str(
        &steps
            .into_iter()
            .map(|x| format!("[{},{}]", x.0, x.1))
            .collect::<Vec<String>>()
            .join(", "),
    );
    result.push_str("]");
    result
}

#[test]
fn test_create_array_string() {
    let steps = vec![(1, 2), (21, 10), (0, 5)];
    let result = create_array_string(steps);
    assert_eq!(result, "[[1,2], [21,10], [0,5]]");
}

pub fn write_solving_steps(image: &PathBuf, map: MonolithMap) -> Result<(), MyError> {
    let solver_filepath = {
        let mut temp = image.clone();
        temp.set_extension("ahk");
        temp
    };
    if solver_filepath.exists() {
        println!(
            "File with solving steps {:?} already exists.",
            solver_filepath
                .file_name()
                .unwrap_or_else(|| std::ffi::OsStr::new("???"))
        );
        return Ok(());
    }

    println!("Solving the map (this may take a minute).");
    let steps = map.solve();

    let solver_file = std::fs::File::create(&solver_filepath)?;

    println!(
        "Writing solving steps to {:?}.",
        solver_filepath
            .file_name()
            .unwrap_or_else(|| std::ffi::OsStr::new("???"))
    );
    let mut writer = BufWriter::new(solver_file);
    let steps_string = create_array_string(steps);
    writer.write_all(
        AHK_TEMPLATE
            .replace("ARRAY_MARKER", &steps_string)
            .as_bytes(),
    )?;

    Ok(())
}
