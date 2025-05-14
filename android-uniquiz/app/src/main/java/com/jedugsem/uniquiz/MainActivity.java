package com.jedugsem.uniquiz;

import android.app.NativeActivity;
import android.content.ClipData;
import android.content.ClipboardManager;
import android.content.Context;
import android.content.Intent;
import android.net.Uri;
import android.os.Bundle;
import android.os.Environment;
import android.provider.Settings;
import android.util.Log;
import android.view.inputmethod.InputMethodManager;
import android.content.Intent;
import android.net.Uri;
import android.provider.Settings;
import android.os.Environment;

public class MainActivity extends NativeActivity {

    private void permissionstorage(){
        Intent intent = new Intent(Settings.ACTION_MANAGE_APP_ALL_FILES_ACCESS_PERMISSION);
        intent.setData(Uri.parse("package:" + getPackageName()));
        startActivity(intent);
    }

    static {
        System.loadLibrary("uniquiz");
    }

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        if (Environment.isExternalStorageManager()) {
            // Do something ...
        }else {
            permissionstorage();

        }

    }

    private void showKeyboard() {
        Log.d("MainActivity", "showKeyboard instance method called");
        InputMethodManager inputManager = getSystemService(InputMethodManager.class);
        inputManager.showSoftInput(getWindow().getDecorView(), InputMethodManager.SHOW_IMPLICIT);
    }

    private void hideKeyboard() {
        Log.d("MainActivity", "hideKeyboard instance method called");
        InputMethodManager inputManager = getSystemService(InputMethodManager.class);
        inputManager.hideSoftInputFromWindow(getWindow().getDecorView().getWindowToken(), 0);
    }

    private String readClipboard() {
        ClipboardManager clipboardManager = (ClipboardManager) getApplicationContext().getSystemService(Context.CLIPBOARD_SERVICE);
        ClipData data = clipboardManager.getPrimaryClip();
        if (data == null) {
            Log.d("MainActivity", "ClipData in readClipboard is null");
            return "";
        }
        ClipData.Item item = data.getItemAt(0);
        if (item == null) {
            Log.d("MainActivity", "Item in readClipboard is null");
            return "";
        }
        return item.coerceToText(this).toString();
    }

    private void writeClipboard(String value) {
        ClipboardManager clipboardManager = (ClipboardManager) getApplicationContext().getSystemService(Context.CLIPBOARD_SERVICE);
        ClipData data = ClipData.newPlainText("MainActivity text", value);
        clipboardManager.setPrimaryClip(data);
    }
}
