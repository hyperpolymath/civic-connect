# IndieWeb2 Bastion - WordPress Integration

WordPress integration for IndieWeb2 Bastion consent management. Allows WordPress users to set consent preferences that are enforced across the IndieWeb2 network.

## Overview

This MU-plugin:
- Adds a user-facing consent preferences page
- Syncs consent to the Bastion consent API
- Supports per-user granular consent controls
- Integrates with existing WordPress user management

## Installation

### Method 1: Must-Use Plugin (Recommended)

```bash
# Copy plugin to wp-content/mu-plugins/
cp indieweb2-consent.php /path/to/wordpress/wp-content/mu-plugins/

# MU-plugins are automatically loaded - no activation needed
```

### Method 2: Regular Plugin

```bash
# Copy to plugins directory
mkdir /path/to/wordpress/wp-content/plugins/indieweb2-consent/
cp indieweb2-consent.php /path/to/wordpress/wp-content/plugins/indieweb2-consent/

# Activate via WordPress admin or WP-CLI
wp plugin activate indieweb2-consent
```

## Configuration

### wp-config.php

Add consent API URL to `wp-config.php`:

```php
// IndieWeb2 Bastion consent API URL
define('INDIEWEB2_CONSENT_API', 'https://bastion.example.com:8082');
```

### WordPress Admin

Navigate to **Settings > IndieWeb2 Consent** to configure:

- **Consent API URL**: Base URL of the consent service
- **Auto-sync**: Enable automatic consent sync on profile updates

Test the connection with the "Test Consent API Connection" button.

## User Interface

### Consent Preferences Page

Users can access their consent preferences at **Consent** in the WordPress admin menu.

Available consent options:

| Setting | Default | Description |
|---------|---------|-------------|
| **Telemetry** | Off | Allow anonymous usage statistics |
| **Search Engine Indexing** | On | Allow search engines to index content |
| **Webmentions** | On | Allow receiving webmentions |
| **DNS Operations** | Off | Allow DNS record operations |

### Identity

The plugin uses the user's website URL or author page as their IndieWeb identity:
- If user has a website URL: `https://example.com/`
- Otherwise: `https://yoursite.com/author/username`

## API Integration

### Consent Sync Flow

1. User saves consent preferences in WordPress
2. Plugin sends POST request to `/consent` endpoint:

```json
{
  "identity": "https://example.com/",
  "telemetry": "off",
  "indexing": "on",
  "webmentions": "on",
  "dnsOperations": "off",
  "timestamp": "2026-01-22T20:00:00Z",
  "source": "wordpress://yoursite.com"
}
```

3. Consent API stores preferences in SurrealDB
4. GraphQL DNS API enforces consent before operations

### Automatic Sync Events

Consent is automatically synced when:
- User saves their consent preferences page
- User registers (new account)
- User profile is updated (if auto-sync enabled)

### Manual Sync

```php
// Programmatically sync consent for a user
$consent = new IndieWeb2_Consent();
$consent->sync_consent($user_id);
```

## Consent Enforcement

The Bastion enforces consent preferences for:

### Webmentions
Only users with `webmentions: on` can receive webmentions.

### DNS Operations
Only users with `dnsOperations: on` can propose DNS mutations via GraphQL API.

### Indexing
Robots.txt and meta tags respect `indexing` preference.

### Telemetry
Anonymous usage stats only collected if `telemetry: on`.

## Hooks and Filters

### Filters

```php
// Customize identity URL generation
add_filter('indieweb2_user_identity', function($identity, $user_id) {
    return 'https://custom-identity.com/users/' . $user_id;
}, 10, 2);

// Customize consent API URL per request
add_filter('indieweb2_consent_api_url', function($url, $user_id) {
    return 'https://regional-api.example.com';
}, 10, 2);

// Modify consent data before sync
add_filter('indieweb2_consent_data', function($consent, $user_id) {
    $consent['customField'] = 'custom-value';
    return $consent;
}, 10, 2);
```

### Actions

```php
// Run after consent sync succeeds
add_action('indieweb2_consent_synced', function($user_id, $response) {
    error_log("Consent synced for user $user_id");
}, 10, 2);

// Run if consent sync fails
add_action('indieweb2_consent_sync_failed', function($user_id, $error) {
    error_log("Consent sync failed for user $user_id: $error");
}, 10, 2);
```

## Privacy & GDPR

This plugin helps with GDPR compliance:

### Right to Access
Users can view their consent preferences anytime via the Consent page.

### Right to Rectification
Users can update preferences anytime.

### Right to Erasure
Administrators can delete user consent:

```php
// Delete consent from local WordPress
delete_user_meta($user_id, 'indieweb2_telemetry');
delete_user_meta($user_id, 'indieweb2_indexing');
delete_user_meta($user_id, 'indieweb2_webmentions');
delete_user_meta($user_id, 'indieweb2_dns_operations');

// Delete from Bastion API
$identity = 'https://example.com/';
$api_url = get_option('indieweb2_consent_api_url');
wp_remote_request($api_url . '/consent/' . urlencode($identity), [
    'method' => 'DELETE',
]);
```

### Right to Data Portability
Consent data is stored in open JSON format and can be exported.

## Troubleshooting

### Connection Failed

**Symptom**: Test connection fails with "Connection failed" error

**Causes**:
- Consent API is not running
- Firewall blocking port 8082
- Incorrect API URL in settings

**Fix**:
```bash
# Check if consent API is running
curl http://localhost:8082/health

# Start consent API if needed
cd services/consent-api
deno run --allow-net --allow-env mod.ts
```

### Consent Not Syncing

**Symptom**: Preferences saved in WordPress but not enforced by Bastion

**Causes**:
- Auto-sync disabled
- API credentials invalid
- Network timeout

**Fix**:
1. Enable auto-sync in Settings > IndieWeb2 Consent
2. Check WordPress error log: `tail -f wp-content/debug.log`
3. Manually trigger sync from profile page

### Identity Mismatch

**Symptom**: Different identity in WordPress vs Bastion

**Cause**: User changed their website URL after initial sync

**Fix**: Re-save consent preferences to sync updated identity

## Development

### Testing

```bash
# Unit tests (requires wp-cli and PHPUnit)
wp scaffold plugin-tests indieweb2-consent
cd wp-content/plugins/indieweb2-consent
phpunit

# Manual testing
wp shell
>>> $consent = new IndieWeb2_Consent();
>>> $consent->sync_consent(1);
```

### Debugging

Enable WordPress debug logging in `wp-config.php`:

```php
define('WP_DEBUG', true);
define('WP_DEBUG_LOG', true);
define('WP_DEBUG_DISPLAY', false);
```

Check logs:
```bash
tail -f wp-content/debug.log
```

## Integration with Other Plugins

### IndieWeb Plugin

```php
// Use IndieWeb plugin's author URL if available
add_filter('indieweb2_user_identity', function($identity, $user_id) {
    if (function_exists('get_author_posts_url')) {
        return get_author_posts_url($user_id);
    }
    return $identity;
}, 10, 2);
```

### Webmention Plugin

```php
// Check consent before processing webmentions
add_filter('webmention_accept', function($accept, $target_url) {
    $user_id = get_user_by_url($target_url);
    $webmentions_enabled = get_user_meta($user_id, 'indieweb2_webmentions', true);
    return $webmentions_enabled === 'on';
}, 10, 2);
```

## Security

### API Authentication

For production deployments, add authentication to consent API requests:

```php
// Add JWT token to requests
add_filter('indieweb2_consent_request_headers', function($headers) {
    $headers['Authorization'] = 'Bearer ' . get_option('indieweb2_jwt_token');
    return $headers;
});
```

### Input Validation

All consent preferences are validated:
- Only `on` or `off` values accepted
- Identity URL must be valid HTTP(S) URL
- Timestamps must be ISO 8601 format

### Output Escaping

All user-facing output is properly escaped:
- `esc_html()` for text
- `esc_attr()` for attributes
- `esc_url()` for URLs

## License

Apache-2.0

## Support

- **Issues**: https://github.com/hyperpolymath/indieweb2-bastion/issues
- **Documentation**: https://indieweb2.hyperpolymath.org/docs/consent
