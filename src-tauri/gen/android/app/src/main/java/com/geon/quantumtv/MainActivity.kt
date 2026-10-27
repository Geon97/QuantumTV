package com.geon.quantumtv

import android.os.Bundle
import android.view.View
import android.view.ViewGroup
import android.view.ViewTreeObserver
import android.webkit.JavascriptInterface
import android.webkit.WebView
import androidx.activity.enableEdgeToEdge
import androidx.core.view.ViewCompat
import androidx.core.view.WindowInsetsCompat

class MainActivity : TauriActivity() {

    @Volatile
    private var bottomInsetCssPx = 0f

    private var appWebView: WebView? = null
    private var isInterfaceAdded = false

    private inner class AndroidInsetsBridge {
        @JavascriptInterface
        fun getBottomInset(): Float = bottomInsetCssPx
    }

    override fun onCreate(savedInstanceState: Bundle?) {
        enableEdgeToEdge()
        super.onCreate(savedInstanceState)

        val decor = window.decorView

        // 1. 监听系统导航栏变化（自动适配：3键虚拟按键通常为 48dp+，手势横条通常为 16-24dp）
        ViewCompat.setOnApplyWindowInsetsListener(decor) { _, insets ->
            val navInsets = insets.getInsets(WindowInsetsCompat.Type.navigationBars())
            val density = resources.displayMetrics.density

            if (density > 0) {
                // 物理像素转 CSS 像素
                bottomInsetCssPx = navInsets.bottom / density
            }
            pushInsetToWebView()
            insets
        }

        // 2. 核心修复：通过 ViewTreeObserver 动态捕获异步创建的 Tauri WebView
        val layoutListener = object : ViewTreeObserver.OnGlobalLayoutListener {
            override fun onGlobalLayout() {
                if (appWebView == null) {
                    val wv = findWebView(decor)
                    if (wv != null) {
                        appWebView = wv
                        attachBridgeAndPush(wv)
                    }
                }
            }
        }
        decor.viewTreeObserver.addOnGlobalLayoutListener(layoutListener)

        // 兜底：延时检测一次
        decor.post {
            val wv = findWebView(decor)
            if (wv != null && appWebView == null) {
                appWebView = wv
                attachBridgeAndPush(wv)
            }
            ViewCompat.requestApplyInsets(decor)
        }
    }

    private fun attachBridgeAndPush(webView: WebView) {
        if (!isInterfaceAdded) {
            webView.addJavascriptInterface(AndroidInsetsBridge(), "AndroidInsets")
            isInterfaceAdded = true
        }

        // 立即读取当前的实际 Inset
        val rootInsets = ViewCompat.getRootWindowInsets(window.decorView)
        if (rootInsets != null) {
            val navInsets = rootInsets.getInsets(WindowInsetsCompat.Type.navigationBars())
            val density = resources.displayMetrics.density
            if (density > 0) {
                bottomInsetCssPx = navInsets.bottom / density
            }
        }
        pushInsetToWebView()
    }

    private fun pushInsetToWebView() {
        val inset = bottomInsetCssPx
        appWebView?.post {
            // 设置 CSS 变量，并广播事件供前端监听
            val script = """
                (function() {
                    document.documentElement.style.setProperty('--app-safe-bottom', '${inset}px');
                    window.dispatchEvent(new CustomEvent('safeBottomInsetChange', { detail: $inset }));
                })();
            """.trimIndent()
            appWebView?.evaluateJavascript(script, null)
        }
    }

    private fun findWebView(view: View): WebView? {
        if (view is WebView) return view

        if (view is ViewGroup) {
            for (i in 0 until view.childCount) {
                val found = findWebView(view.getChildAt(i))
                if (found != null) return found
            }
        }
        return null
    }
}