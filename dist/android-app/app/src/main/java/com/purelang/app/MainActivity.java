package com.purelang.app;

import android.app.Activity;
import android.os.Bundle;
import android.widget.Button;
import android.widget.LinearLayout;
import android.widget.TextView;
import android.widget.Toast;

/**
 * Host Activity for PureLang native code (libpureapp.so).
 * Loads the shared library when present and exposes a simple UI shell.
 */
public class MainActivity extends Activity {
    static {
        try {
            System.loadLibrary("pureapp");
        } catch (UnsatisfiedLinkError e) {
            // Library optional until NDK build is run
        }
    }

    /** Optional JNI entry from purec-generated code */
    public static native long purelangMain();

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        LinearLayout root = new LinearLayout(this);
        root.setOrientation(LinearLayout.VERTICAL);
        root.setPadding(48, 48, 48, 48);

        TextView title = new TextView(this);
        title.setText("PureLang Android Host");
        title.setTextSize(22f);
        root.addView(title);

        TextView status = new TextView(this);
        status.setText("Native lib: " + (hasNative() ? "loaded" : "not linked yet"));
        status.setTextSize(16f);
        status.setPadding(0, 24, 0, 24);
        root.addView(status);

        Button run = new Button(this);
        run.setText("Run purelangMain()");
        run.setOnClickListener(v -> {
            try {
                long r = purelangMain();
                Toast.makeText(this, "purelangMain → " + r, Toast.LENGTH_SHORT).show();
                status.setText("Last result: " + r);
            } catch (UnsatisfiedLinkError e) {
                Toast.makeText(this, "Native method missing — build libpureapp.so", Toast.LENGTH_LONG).show();
            }
        });
        root.addView(run);

        setContentView(root);
    }

    private boolean hasNative() {
        try {
            System.mapLibraryName("pureapp");
            return true;
        } catch (Throwable t) {
            return false;
        }
    }
}
