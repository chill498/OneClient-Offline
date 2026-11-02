// Folder-local launcher for the modified OneClient build.
// No registry or persistent environment settings are changed.
#include <windows.h>
#include <wchar.h>

int WINAPI wWinMain(HINSTANCE, HINSTANCE, PWSTR, int) {
    wchar_t folder[32768];
    DWORD length = GetModuleFileNameW(nullptr, folder, 32768);
    if (!length || length >= 32768) return 1;
    wchar_t* separator = wcsrchr(folder, L'\\');
    if (!separator) return 1;
    *separator = L'\0';

    wchar_t executable[32768];
    wchar_t profile[32768];
    wchar_t command[32768];
    if (wcscpy_s(executable, folder) || wcscat_s(executable, L"\\oneclient_app.exe") ||
        wcscpy_s(profile, folder) || wcscat_s(profile, L"\\data") ||
        wcscpy_s(command, L"\"") || wcscat_s(command, executable) || wcscat_s(command, L"\"")) return 1;

    if (!SetEnvironmentVariableW(L"ONECLIENT_HOME", profile)) return GetLastError();
    STARTUPINFOW startup = {};
    startup.cb = sizeof(startup);
    PROCESS_INFORMATION process = {};
    if (!CreateProcessW(executable, command, nullptr, nullptr, FALSE, 0, nullptr,
                        folder, &startup, &process)) {
        DWORD error = GetLastError();
        MessageBoxW(nullptr,
                    L"Could not open OneClient. Extract the whole ZIP and keep this launcher beside oneclient_app.exe and the DLL files.",
                    L"OneClient Offline", MB_OK | MB_ICONERROR);
        return error;
    }
    CloseHandle(process.hThread);
    CloseHandle(process.hProcess);
    return 0;
}
