<?php
/**
 * Plugin Name: IndieWeb2 Bastion Consent
 * Plugin URI: https://github.com/hyperpolymath/indieweb2-bastion
 * Description: Sends user consent preferences to IndieWeb2 Bastion consent API
 * Version: 0.1.0
 * Author: Hyperpolymath
 * Author URI: https://hyperpolymath.org
 * License: PMPL-1.0-or-later
 * SPDX-License-Identifier: PMPL-1.0-or-later
 */

if (!defined('ABSPATH')) {
    exit; // Exit if accessed directly
}

class IndieWeb2_Consent {
    private $consent_api_url;
    private $settings_page_slug = 'indieweb2-consent';

    public function __construct() {
        $this->consent_api_url = defined('INDIEWEB2_CONSENT_API')
            ? INDIEWEB2_CONSENT_API
            : 'http://localhost:8082';

        add_action('admin_menu', [$this, 'add_settings_page']);
        add_action('admin_init', [$this, 'register_settings']);
        add_action('profile_update', [$this, 'sync_consent_on_profile_update'], 10, 2);
        add_action('user_register', [$this, 'sync_consent_on_registration']);
        add_action('wp_ajax_indieweb2_test_connection', [$this, 'ajax_test_connection']);
    }

    /**
     * Add settings page to WordPress admin
     */
    public function add_settings_page() {
        add_options_page(
            'IndieWeb2 Consent Settings',
            'IndieWeb2 Consent',
            'manage_options',
            $this->settings_page_slug,
            [$this, 'render_settings_page']
        );

        add_menu_page(
            'IndieWeb2 Consent',
            'Consent',
            'read',
            'indieweb2-user-consent',
            [$this, 'render_user_consent_page'],
            'dashicons-shield',
            75
        );
    }

    /**
     * Register plugin settings
     */
    public function register_settings() {
        register_setting('indieweb2_consent', 'indieweb2_consent_api_url');
        register_setting('indieweb2_consent', 'indieweb2_consent_auto_sync');

        // Per-user consent settings
        add_user_meta_field('indieweb2_telemetry', 'off');
        add_user_meta_field('indieweb2_indexing', 'on');
        add_user_meta_field('indieweb2_webmentions', 'on');
        add_user_meta_field('indieweb2_dns_operations', 'off');
    }

    /**
     * Render admin settings page
     */
    public function render_settings_page() {
        if (!current_user_can('manage_options')) {
            return;
        }

        ?>
        <div class="wrap">
            <h1><?php echo esc_html(get_admin_page_title()); ?></h1>
            <form action="options.php" method="post">
                <?php
                settings_fields('indieweb2_consent');
                do_settings_sections('indieweb2_consent');
                ?>

                <table class="form-table">
                    <tr>
                        <th scope="row">
                            <label for="indieweb2_consent_api_url">Consent API URL</label>
                        </th>
                        <td>
                            <input
                                type="url"
                                id="indieweb2_consent_api_url"
                                name="indieweb2_consent_api_url"
                                value="<?php echo esc_attr(get_option('indieweb2_consent_api_url', $this->consent_api_url)); ?>"
                                class="regular-text"
                            />
                            <p class="description">URL of the IndieWeb2 Bastion consent API</p>
                        </td>
                    </tr>
                    <tr>
                        <th scope="row">
                            <label for="indieweb2_consent_auto_sync">Auto-sync on profile update</label>
                        </th>
                        <td>
                            <input
                                type="checkbox"
                                id="indieweb2_consent_auto_sync"
                                name="indieweb2_consent_auto_sync"
                                value="1"
                                <?php checked(get_option('indieweb2_consent_auto_sync', '1'), '1'); ?>
                            />
                            <p class="description">Automatically sync consent preferences when user profile is updated</p>
                        </td>
                    </tr>
                </table>

                <?php submit_button(); ?>
            </form>

            <h2>Connection Test</h2>
            <button type="button" id="test-connection" class="button">Test Consent API Connection</button>
            <div id="test-result" style="margin-top: 10px;"></div>

            <script>
            jQuery(document).ready(function($) {
                $('#test-connection').on('click', function() {
                    var $button = $(this);
                    var $result = $('#test-result');

                    $button.prop('disabled', true).text('Testing...');
                    $result.html('');

                    $.ajax({
                        url: ajaxurl,
                        method: 'POST',
                        data: {
                            action: 'indieweb2_test_connection',
                            _ajax_nonce: '<?php echo wp_create_nonce('indieweb2_test_connection'); ?>'
                        },
                        success: function(response) {
                            if (response.success) {
                                $result.html('<p style="color: green;">✓ Connection successful!</p>');
                            } else {
                                $result.html('<p style="color: red;">✗ Connection failed: ' + response.data + '</p>');
                            }
                        },
                        error: function() {
                            $result.html('<p style="color: red;">✗ AJAX request failed</p>');
                        },
                        complete: function() {
                            $button.prop('disabled', false).text('Test Consent API Connection');
                        }
                    });
                });
            });
            </script>
        </div>
        <?php
    }

    /**
     * Render user consent preferences page
     */
    public function render_user_consent_page() {
        $user_id = get_current_user_id();

        if ($_SERVER['REQUEST_METHOD'] === 'POST' && check_admin_referer('indieweb2_consent_save')) {
            update_user_meta($user_id, 'indieweb2_telemetry', $_POST['indieweb2_telemetry'] ?? 'off');
            update_user_meta($user_id, 'indieweb2_indexing', $_POST['indieweb2_indexing'] ?? 'off');
            update_user_meta($user_id, 'indieweb2_webmentions', $_POST['indieweb2_webmentions'] ?? 'off');
            update_user_meta($user_id, 'indieweb2_dns_operations', $_POST['indieweb2_dns_operations'] ?? 'off');

            $this->sync_consent($user_id);
            echo '<div class="notice notice-success"><p>Consent preferences saved and synced to IndieWeb2 Bastion.</p></div>';
        }

        $telemetry = get_user_meta($user_id, 'indieweb2_telemetry', true) ?: 'off';
        $indexing = get_user_meta($user_id, 'indieweb2_indexing', true) ?: 'on';
        $webmentions = get_user_meta($user_id, 'indieweb2_webmentions', true) ?: 'on';
        $dns_operations = get_user_meta($user_id, 'indieweb2_dns_operations', true) ?: 'off';

        ?>
        <div class="wrap">
            <h1>My Consent Preferences</h1>
            <p>Control how your data is used by IndieWeb2 services.</p>

            <form method="post">
                <?php wp_nonce_field('indieweb2_consent_save'); ?>

                <table class="form-table">
                    <tr>
                        <th scope="row">Telemetry</th>
                        <td>
                            <select name="indieweb2_telemetry">
                                <option value="off" <?php selected($telemetry, 'off'); ?>>Off</option>
                                <option value="on" <?php selected($telemetry, 'on'); ?>>On</option>
                            </select>
                            <p class="description">Allow collection of anonymous usage statistics</p>
                        </td>
                    </tr>
                    <tr>
                        <th scope="row">Search Engine Indexing</th>
                        <td>
                            <select name="indieweb2_indexing">
                                <option value="off" <?php selected($indexing, 'off'); ?>>Off</option>
                                <option value="on" <?php selected($indexing, 'on'); ?>>On</option>
                            </select>
                            <p class="description">Allow search engines to index your content</p>
                        </td>
                    </tr>
                    <tr>
                        <th scope="row">Webmentions</th>
                        <td>
                            <select name="indieweb2_webmentions">
                                <option value="off" <?php selected($webmentions, 'off'); ?>>Off</option>
                                <option value="on" <?php selected($webmentions, 'on'); ?>>On</option>
                            </select>
                            <p class="description">Allow receiving webmentions from other sites</p>
                        </td>
                    </tr>
                    <tr>
                        <th scope="row">DNS Operations</th>
                        <td>
                            <select name="indieweb2_dns_operations">
                                <option value="off" <?php selected($dns_operations, 'off'); ?>>Off</option>
                                <option value="on" <?php selected($dns_operations, 'on'); ?>>On</option>
                            </select>
                            <p class="description">Allow DNS record operations on your behalf</p>
                        </td>
                    </tr>
                </table>

                <?php submit_button('Save Consent Preferences'); ?>
            </form>

            <h2>Your Identity</h2>
            <p><strong>Identity URL:</strong> <?php echo esc_html($this->get_user_identity($user_id)); ?></p>
            <p><em>This URL is used to identify you in the IndieWeb2 network.</em></p>

            <h2>Data Rights</h2>
            <ul>
                <li><strong>Access:</strong> You can request a copy of your data anytime</li>
                <li><strong>Portability:</strong> Your data is stored in open formats</li>
                <li><strong>Erasure:</strong> You can request deletion of your consent records</li>
                <li><strong>Rectification:</strong> You can update your preferences anytime</li>
            </ul>
        </div>
        <?php
    }

    /**
     * Get user identity URL
     */
    private function get_user_identity($user_id) {
        $user = get_userdata($user_id);
        $site_url = get_site_url();
        return $user->user_url ?: $site_url . '/author/' . $user->user_nicename;
    }

    /**
     * Sync consent to bastion API
     */
    public function sync_consent($user_id) {
        $identity = $this->get_user_identity($user_id);

        $consent = [
            'identity' => $identity,
            'telemetry' => get_user_meta($user_id, 'indieweb2_telemetry', true) ?: 'off',
            'indexing' => get_user_meta($user_id, 'indieweb2_indexing', true) ?: 'on',
            'webmentions' => get_user_meta($user_id, 'indieweb2_webmentions', true) ?: 'on',
            'dnsOperations' => get_user_meta($user_id, 'indieweb2_dns_operations', true) ?: 'off',
            'timestamp' => gmdate('c'),
            'source' => 'wordpress://' . parse_url(get_site_url(), PHP_URL_HOST),
        ];

        $api_url = get_option('indieweb2_consent_api_url', $this->consent_api_url);

        $response = wp_remote_post($api_url . '/consent', [
            'headers' => ['Content-Type' => 'application/json'],
            'body' => json_encode($consent),
            'timeout' => 10,
        ]);

        if (is_wp_error($response)) {
            error_log('IndieWeb2 Consent API error: ' . $response->get_error_message());
            return false;
        }

        $status_code = wp_remote_retrieve_response_code($response);
        if ($status_code !== 200 && $status_code !== 201) {
            error_log('IndieWeb2 Consent API returned status ' . $status_code);
            return false;
        }

        return true;
    }

    /**
     * Sync consent on profile update
     */
    public function sync_consent_on_profile_update($user_id, $old_user_data) {
        if (get_option('indieweb2_consent_auto_sync', '1') === '1') {
            $this->sync_consent($user_id);
        }
    }

    /**
     * Sync consent on user registration
     */
    public function sync_consent_on_registration($user_id) {
        // Set default consent preferences for new users
        update_user_meta($user_id, 'indieweb2_telemetry', 'off');
        update_user_meta($user_id, 'indieweb2_indexing', 'on');
        update_user_meta($user_id, 'indieweb2_webmentions', 'on');
        update_user_meta($user_id, 'indieweb2_dns_operations', 'off');

        $this->sync_consent($user_id);
    }

    /**
     * AJAX handler for connection test
     */
    public function ajax_test_connection() {
        check_ajax_referer('indieweb2_test_connection');

        $api_url = get_option('indieweb2_consent_api_url', $this->consent_api_url);

        $response = wp_remote_get($api_url . '/health', ['timeout' => 5]);

        if (is_wp_error($response)) {
            wp_send_json_error($response->get_error_message());
        }

        $status_code = wp_remote_retrieve_response_code($response);
        if ($status_code === 200) {
            wp_send_json_success('Connection successful');
        } else {
            wp_send_json_error('Unexpected status code: ' . $status_code);
        }
    }
}

// Initialize plugin
new IndieWeb2_Consent();
