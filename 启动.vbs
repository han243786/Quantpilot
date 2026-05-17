Set shell = CreateObject("WScript.Shell")
Set fso = CreateObject("Scripting.FileSystemObject")

dir = fso.GetParentFolderName(WScript.ScriptFullName)

shell.CurrentDirectory = dir

shell.Environment("Process")("QUANTPILOT_DEV") = "true"

shell.Run "quantpilot.exe", 0, False

WScript.Sleep 8000

shell.Run "http://127.0.0.1:3000"

MsgBox "QuantPilot started at http://127.0.0.1:3000" & vbCrLf & "Press OK to stop server.", vbInformation, "QuantPilot"

shell.Run "taskkill /f /im quantpilot.exe", 0, True
