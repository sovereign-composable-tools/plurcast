// Migration script to move keys from old format to new multi-account format
// This fixes the "nostr keys file not found" error after rebuilding

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Plurcast Credential Migration Tool");
    println!("==================================\n");

    let platforms = vec![
        ("nostr", "private_key"),
        ("mastodon", "access_token"),
        ("bluesky", "app_password"),
    ];

    let mut migrated = 0;
    let mut skipped = 0;
    let mut errors = 0;

    for (platform, key_type) in platforms {
        let old_service = format!("plurcast.{}", platform);
        let new_service = format!("plurcast.{}.default", platform);

        print!("Checking {}...", platform);

        // Check if old format exists
        let old_entry = match keyring::Entry::new(&old_service, key_type) {
            Ok(e) => e,
            Err(e) => {
                println!(" SKIP (keyring error: {})", e);
                errors += 1;
                continue;
            }
        };

        let old_value = match old_entry.get_password() {
            Ok(v) => v,
            Err(keyring::Error::NoEntry) => {
                println!(" SKIP (not found in old format)");
                skipped += 1;
                continue;
            }
            Err(e) => {
                println!(" ERROR (failed to read: {})", e);
                errors += 1;
                continue;
            }
        };

        // Check if new format already exists
        let new_entry = match keyring::Entry::new(&new_service, key_type) {
            Ok(e) => e,
            Err(e) => {
                println!(" ERROR (keyring error: {})", e);
                errors += 1;
                continue;
            }
        };

        if new_entry.get_password().is_ok() {
            println!(" SKIP (already migrated)");
            skipped += 1;
            continue;
        }

        // Migrate: copy old value to new location
        match new_entry.set_password(&old_value) {
            Ok(_) => {
                // Verify the migration
                match new_entry.get_password() {
                    Ok(verified) if verified == old_value => {
                        println!(" ✓ MIGRATED");
                        migrated += 1;
                    }
                    Ok(_) => {
                        println!(" ERROR (verification failed)");
                        errors += 1;
                    }
                    Err(e) => {
                        println!(" ERROR (verification failed: {})", e);
                        errors += 1;
                    }
                }
            }
            Err(e) => {
                println!(" ERROR (failed to set: {})", e);
                errors += 1;
            }
        }
    }

    println!("\nMigration Summary:");
    println!("  Migrated: {}", migrated);
    println!("  Skipped:  {}", skipped);
    println!("  Errors:   {}", errors);

    if migrated > 0 {
        println!("\n✓ Migration successful! Your credentials are now in the new format.");
        println!("  You can now use plur-post and other commands.");
    } else if errors > 0 {
        println!("\n⚠ Some errors occurred. Check the output above.");
    } else {
        println!("\n→ No migration needed. All credentials are up to date.");
    }

    Ok(())
}
