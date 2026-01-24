/**
 * NGA Server
 * Handles cookie-based session authentication and serves static files
 */

const express = require('express');
const session = require('express-session');
const cookieParser = require('cookie-parser');
const crypto = require('crypto');
const path = require('path');

const app = express();

// Configuration
const PORT = process.env.PORT || process.env.AUTH_PORT || 3001;
const SESSION_TIMEOUT_MS = 60 * 60 * 1000; // 1 hour in milliseconds
const PASSWORD = process.env.HTTP_AUTH_PASS;
const SESSION_SECRET = process.env.SESSION_SECRET || crypto.randomBytes(32).toString('hex');
const IS_PRODUCTION = process.env.NODE_ENV === 'production';
const STATIC_DIR = __dirname;

// Trust proxy for Heroku
app.set('trust proxy', 1);

// Middleware
app.use(express.json());
app.use(express.urlencoded({ extended: true }));
app.use(cookieParser());

// Session configuration
app.use(session({
    name: 'nga_session',
    secret: SESSION_SECRET,
    resave: false,
    saveUninitialized: false,
    cookie: {
        httpOnly: true,
        secure: IS_PRODUCTION, // HTTPS only in production
        sameSite: 'lax', // Changed from 'strict' for better compatibility
        maxAge: SESSION_TIMEOUT_MS
    }
}));

/**
 * POST /api/login
 * Verify password and create session
 */
app.post('/api/login', (req, res) => {
    const { password } = req.body;

    // Check if password protection is enabled
    if (!PASSWORD) {
        return res.status(500).json({
            success: false,
            error: 'Password protection not configured'
        });
    }

    // Verify password
    if (password === PASSWORD) {
        // Create session
        req.session.authenticated = true;
        req.session.loginTime = Date.now();
        req.session.expiresAt = Date.now() + SESSION_TIMEOUT_MS;

        res.json({
            success: true,
            expiresAt: req.session.expiresAt,
            timeoutMs: SESSION_TIMEOUT_MS
        });
    } else {
        // Invalid password
        res.status(401).json({
            success: false,
            error: 'Invalid password'
        });
    }
});

/**
 * GET /api/verify
 * Check if session is valid (used by nginx auth_request)
 * Returns 200 if authenticated, 401 if not
 */
app.get('/api/verify', (req, res) => {
    // Check if password protection is disabled
    if (!PASSWORD) {
        return res.status(200).send('OK');
    }

    // Check session
    if (req.session && req.session.authenticated) {
        const now = Date.now();
        
        // Check if session has expired
        if (req.session.expiresAt && now > req.session.expiresAt) {
            req.session.destroy();
            return res.status(401).send('Session expired');
        }

        return res.status(200).send('OK');
    }

    res.status(401).send('Unauthorized');
});

/**
 * POST /api/logout
 * Destroy session and clear cookie
 */
app.post('/api/logout', (req, res) => {
    req.session.destroy((err) => {
        if (err) {
            console.error('Error destroying session:', err);
            return res.status(500).json({
                success: false,
                error: 'Failed to logout'
            });
        }

        res.clearCookie('nga_session');
        res.json({ success: true });
    });
});

/**
 * GET /api/session
 * Return session info (time remaining, etc.)
 */
app.get('/api/session', (req, res) => {
    // Check if password protection is disabled
    if (!PASSWORD) {
        return res.json({
            authenticated: true,
            passwordProtectionEnabled: false
        });
    }

    if (req.session && req.session.authenticated) {
        const now = Date.now();
        const timeRemaining = Math.max(0, req.session.expiresAt - now);
        
        // Check if session has expired
        if (timeRemaining === 0) {
            req.session.destroy();
            return res.json({
                authenticated: false,
                expired: true
            });
        }

        res.json({
            authenticated: true,
            passwordProtectionEnabled: true,
            loginTime: req.session.loginTime,
            expiresAt: req.session.expiresAt,
            timeRemainingMs: timeRemaining,
            timeRemainingMinutes: Math.ceil(timeRemaining / 60000)
        });
    } else {
        res.json({
            authenticated: false,
            passwordProtectionEnabled: true
        });
    }
});

/**
 * Health check endpoint
 */
app.get('/api/health', (req, res) => {
    res.json({ status: 'ok', timestamp: Date.now() });
});

// ============================================================================
// STATIC FILE SERVING WITH AUTHENTICATION
// ============================================================================

/**
 * Authentication middleware for protected routes
 */
function requireAuth(req, res, next) {
    // Skip auth if password protection is disabled
    if (!PASSWORD) {
        return next();
    }

    // Check session
    if (req.session && req.session.authenticated) {
        const now = Date.now();
        
        // Check if session has expired
        if (req.session.expiresAt && now > req.session.expiresAt) {
            req.session.destroy();
            return res.redirect('/login.html');
        }
        
        return next();
    }

    // Not authenticated - redirect to login
    res.redirect('/login.html');
}

/**
 * Public routes (no authentication required)
 */
// Login page
app.get('/login.html', (req, res) => {
    res.sendFile(path.join(STATIC_DIR, 'login.html'));
});

// Styles (needed for login page)
app.get('/styles.css', (req, res) => {
    res.set('Cache-Control', 'public, max-age=3600');
    res.sendFile(path.join(STATIC_DIR, 'styles.css'));
});

/**
 * Protected static files
 */
// WASM files with correct MIME type
app.get('/wasm/*.wasm', requireAuth, (req, res) => {
    const filePath = path.join(STATIC_DIR, req.path);
    res.set('Content-Type', 'application/wasm');
    res.set('Cache-Control', 'public, max-age=2592000, immutable');
    res.sendFile(filePath);
});

// Other WASM directory files (JS, etc.)
app.get('/wasm/*', requireAuth, (req, res) => {
    const filePath = path.join(STATIC_DIR, req.path);
    res.set('Cache-Control', 'public, max-age=3600');
    res.sendFile(filePath);
});

// Main app and other protected files
app.get('/', requireAuth, (req, res) => {
    res.sendFile(path.join(STATIC_DIR, 'index.html'));
});

app.get('/index.html', requireAuth, (req, res) => {
    res.sendFile(path.join(STATIC_DIR, 'index.html'));
});

// Protected static files (JS, JSON, YAML, MD, etc.)
app.get('/*.js', requireAuth, (req, res) => {
    res.set('Cache-Control', 'public, max-age=3600');
    res.sendFile(path.join(STATIC_DIR, req.path));
});

app.get('/*.json', requireAuth, (req, res) => {
    res.set('Cache-Control', 'public, max-age=3600');
    res.set('Content-Type', 'application/json');
    res.sendFile(path.join(STATIC_DIR, req.path));
});

app.get('/*.yaml', requireAuth, (req, res) => {
    res.set('Cache-Control', 'public, max-age=3600');
    res.sendFile(path.join(STATIC_DIR, req.path));
});

app.get('/*.yml', requireAuth, (req, res) => {
    res.set('Cache-Control', 'public, max-age=3600');
    res.sendFile(path.join(STATIC_DIR, req.path));
});

app.get('/*.md', requireAuth, (req, res) => {
    res.set('Cache-Control', 'public, max-age=3600');
    res.sendFile(path.join(STATIC_DIR, req.path));
});

// Catch-all for other protected static files
app.get('/*', requireAuth, (req, res, next) => {
    const filePath = path.join(STATIC_DIR, req.path);
    res.sendFile(filePath, (err) => {
        if (err) {
            // File not found, let it fall through to 404
            next();
        }
    });
});

// 404 handler
app.use((req, res) => {
    res.status(404).send('Not Found');
});

// Start server
app.listen(PORT, () => {
    console.log(`NGA Server running on port ${PORT}`);
    console.log(`Session timeout: ${SESSION_TIMEOUT_MS / 60000} minutes`);
    console.log(`Password protection: ${PASSWORD ? 'enabled' : 'disabled'}`);
    console.log(`Static files: ${STATIC_DIR}`);
});
