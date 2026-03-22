//! Embedded admin dashboard HTML

/// Render the admin dashboard HTML page
pub fn render_dashboard(app_name: &str, app_version: &str) -> String {
    format!(r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{app_name} - Admin Dashboard</title>
    <style>
        * {{ box-sizing: border-box; margin: 0; padding: 0; }}
        body {{ font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; background: #0f172a; color: #e2e8f0; min-height: 100vh; }}
        .header {{ background: #1e293b; border-bottom: 1px solid #334155; padding: 1rem 2rem; display: flex; align-items: center; justify-content: space-between; }}
        .header h1 {{ font-size: 1.25rem; font-weight: 600; color: #f1f5f9; }}
        .header .badge {{ background: #3b82f6; color: white; padding: 0.25rem 0.75rem; border-radius: 9999px; font-size: 0.75rem; font-weight: 500; }}
        .container {{ max-width: 1200px; margin: 2rem auto; padding: 0 1.5rem; }}
        .grid {{ display: grid; grid-template-columns: repeat(auto-fit, minmax(240px, 1fr)); gap: 1.5rem; margin-bottom: 2rem; }}
        .card {{ background: #1e293b; border: 1px solid #334155; border-radius: 0.75rem; padding: 1.5rem; }}
        .card-title {{ font-size: 0.875rem; font-weight: 500; color: #94a3b8; text-transform: uppercase; letter-spacing: 0.05em; margin-bottom: 0.75rem; }}
        .card-value {{ font-size: 2rem; font-weight: 700; color: #f1f5f9; }}
        .card-value.green {{ color: #34d399; }}
        .card-value.blue {{ color: #60a5fa; }}
        .card-value.yellow {{ color: #fbbf24; }}
        .card-value.red {{ color: #f87171; }}
        .card-subtitle {{ font-size: 0.75rem; color: #64748b; margin-top: 0.5rem; }}
        .section {{ background: #1e293b; border: 1px solid #334155; border-radius: 0.75rem; padding: 1.5rem; margin-bottom: 1.5rem; }}
        .section-title {{ font-size: 1rem; font-weight: 600; color: #f1f5f9; margin-bottom: 1rem; padding-bottom: 0.75rem; border-bottom: 1px solid #334155; }}
        .feature-grid {{ display: flex; flex-wrap: wrap; gap: 0.5rem; }}
        .feature-badge {{ background: #0f172a; border: 1px solid #3b82f6; color: #60a5fa; padding: 0.25rem 0.75rem; border-radius: 9999px; font-size: 0.75rem; font-weight: 500; }}
        .system-row {{ display: flex; justify-content: space-between; padding: 0.5rem 0; border-bottom: 1px solid #1e293b; }}
        .system-row:last-child {{ border-bottom: none; }}
        .system-key {{ color: #94a3b8; font-size: 0.875rem; }}
        .system-val {{ color: #e2e8f0; font-size: 0.875rem; font-weight: 500; }}
        .status-dot {{ display: inline-block; width: 8px; height: 8px; background: #34d399; border-radius: 50%; margin-right: 0.5rem; animation: pulse 2s infinite; }}
        @keyframes pulse {{ 0%, 100% {{ opacity: 1; }} 50% {{ opacity: 0.5; }} }}
        .refresh-btn {{ background: #3b82f6; color: white; border: none; padding: 0.5rem 1rem; border-radius: 0.375rem; cursor: pointer; font-size: 0.875rem; font-weight: 500; }}
        .refresh-btn:hover {{ background: #2563eb; }}
        .loading {{ color: #64748b; font-style: italic; }}
        .error-msg {{ color: #f87171; }}
    </style>
</head>
<body>
    <div class="header">
        <h1>&#9889; {app_name} <span style="color:#64748b;font-weight:400">Admin Dashboard</span></h1>
        <div style="display:flex;align-items:center;gap:1rem">
            <span class="badge">v{app_version}</span>
            <button class="refresh-btn" onclick="loadStats()">&#8635; Refresh</button>
        </div>
    </div>
    <div class="container">
        <div class="grid">
            <div class="card">
                <div class="card-title">Status</div>
                <div class="card-value green"><span class="status-dot"></span>Healthy</div>
                <div class="card-subtitle">All systems operational</div>
            </div>
            <div class="card">
                <div class="card-title">Uptime</div>
                <div class="card-value blue" id="uptime">&#8212;</div>
                <div class="card-subtitle" id="uptime-seconds">Loading...</div>
            </div>
            <div class="card">
                <div class="card-title">Total Requests</div>
                <div class="card-value" id="total-requests">&#8212;</div>
                <div class="card-subtitle">Since startup</div>
            </div>
            <div class="card">
                <div class="card-title">Error Rate</div>
                <div class="card-value" id="error-rate">&#8212;</div>
                <div class="card-subtitle" id="error-count">Loading...</div>
            </div>
        </div>

        <div class="section">
            <div class="section-title">System Information</div>
            <div id="system-info"><div class="loading">Loading system info...</div></div>
        </div>

        <div class="section">
            <div class="section-title">Enabled Features</div>
            <div class="feature-grid" id="features"><div class="loading">Loading features...</div></div>
        </div>

        <div class="section">
            <div class="section-title">Quick Links</div>
            <div style="display:flex;gap:1rem;flex-wrap:wrap">
                <a href="/health" style="color:#60a5fa;text-decoration:none;font-size:0.875rem">Health Check</a>
                <a href="/docs" style="color:#60a5fa;text-decoration:none;font-size:0.875rem">API Docs</a>
                <a href="/metrics" style="color:#60a5fa;text-decoration:none;font-size:0.875rem">Metrics</a>
                <a href="/graphql/playground" style="color:#60a5fa;text-decoration:none;font-size:0.875rem">GraphQL Playground</a>
            </div>
        </div>
    </div>

    <script>
        async function loadStats() {{
            try {{
                const res = await fetch('./stats');
                const data = await res.json();

                document.getElementById('uptime').textContent = data.uptime_human || '—';
                document.getElementById('uptime-seconds').textContent = data.uptime_seconds + 's total';
                document.getElementById('total-requests').textContent = data.total_requests.toLocaleString();

                const errRate = data.error_rate.toFixed(2) + '%';
                const errEl = document.getElementById('error-rate');
                errEl.textContent = errRate;
                errEl.className = 'card-value ' + (data.error_rate > 5 ? 'red' : data.error_rate > 1 ? 'yellow' : 'green');
                document.getElementById('error-count').textContent = data.total_errors + ' errors total';

                const sysEl = document.getElementById('system-info');
                const sys = data.system;
                sysEl.innerHTML =
                    '<div class="system-row"><span class="system-key">OS</span><span class="system-val">' + sys.os + '</span></div>' +
                    '<div class="system-row"><span class="system-key">Architecture</span><span class="system-val">' + sys.arch + '</span></div>' +
                    '<div class="system-row"><span class="system-key">rapid-rs Version</span><span class="system-val">' + sys.rapid_rs_version + '</span></div>';

                const featEl = document.getElementById('features');
                featEl.innerHTML = sys.features.map(function(f) {{
                    return '<span class="feature-badge">' + f + '</span>';
                }}).join('');
            }} catch(e) {{
                document.getElementById('uptime').innerHTML = '<span class="error-msg">Error loading stats</span>';
            }}
        }}

        loadStats();
        setInterval(loadStats, 30000);
    </script>
</body>
</html>"#, app_name = app_name, app_version = app_version)
}
