package com.purelang.app;
import android.app.Activity;
import android.os.Bundle;
import android.widget.TextView;
public class MainActivity extends Activity {
  static { try { System.loadLibrary("pureapp"); } catch (Throwable t) {} }
  @Override protected void onCreate(Bundle b) {
    super.onCreate(b);
    TextView tv = new TextView(this);
    tv.setText("PureLang Android host");
    tv.setTextSize(20f);
    setContentView(tv);
  }
}
