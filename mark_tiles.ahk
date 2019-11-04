#SingleInstance Force

tiles := []

~q::
    output := "["
    for k,v in tiles
        output .= "[" . v[1] . "," . v[2] . "],"
    output = % SubStr(output, 1, -1)
    output .= "]"
    FileDelete, tiles.txt
    FileAppend, %output%, tiles.txt

    ExitApp
    Return

~Space::
    MouseGetPos, x, y
    tx := Floor((x - 80) / 80)
    ty := Floor((y - 80) / 80)
    tiles.push([tx, ty])
