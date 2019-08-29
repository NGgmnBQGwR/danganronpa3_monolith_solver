# Automatic solver for Monolith minigame from Danganronpa V3, in Rust.

Make sure that you play in 1980x1080, borderless, you just started the minigame on Mean difficulty and the cursor isn't obstucting any tiles.

Make a screenshot of the Monolith minigame game field.
Save it as PNG and put it in the same folder as monolith_solver executable.

Run monolith_solver executable.
Near your screenshot file, a new "*.map" file should have appeared. If not, consult error messages.
Near your screenshot file, a new "*.ahk" file should have appeared. If not, consult error messages.

Run the resulting ".ahk" file. Make sure you have Autohotkey (https://www.autohotkey.com/) installed.

Return to the game.
You should see green circle indicating your next move. If not, consult error messages in autohotkey console.
Left mouse click shows the next step, right mouse click shows the previous step.
When done, press "q" to close the AHK script.

In case you need it, list of tiles to click is stored in ahk file as "tiles" variable.
