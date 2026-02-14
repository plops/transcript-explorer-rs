import sqlite_minutils.db as db
import struct
from loguru import logger
import os
import sys

def cleanup_database(input_db_path, output_db_path):
    if not os.path.exists(input_db_path):
        logger.error(f"Input database not found: {input_db_path}")
        return

    logger.info(f"Cleaning up {input_db_path} -> {output_db_path}")

    # Copy the database to output path first to preserve schema and other tables
    if os.path.exists(output_db_path):
        logger.warning(f"Output path {output_db_path} already exists. Overwriting.")
        os.remove(output_db_path)
    
    import shutil
    shutil.copy2(input_db_path, output_db_path)

    database = db.Database(output_db_path)
    items = database["items"]

    # 1. Filter out error messages
    # The user mentioned summary like 'Error: value error' or 'Error: resource exhausted'
    logger.info("Removing rows with error messages in summary...")
    rows_before = items.count
    
    # We can delete rows where summary starts with 'Error:' or contains it in a way that looks like an error
    # Based on user's screenshot, it's often 'Error: ...'
    database.execute("DELETE FROM items WHERE summary LIKE 'Error:%' OR summary LIKE '% Error: %'")
    
    rows_after = items.count
    logger.info(f"Removed {rows_before - rows_after} error rows. {rows_after} rows remaining.")

    # 2. Empty out large text columns and truncate embeddings
    logger.info("Emptying large text columns and truncating embeddings...")
    
    columns_to_empty = [
        "transcript", 
        "timestamps", 
        "timestamped_summary_in_youtube_format"
    ]
    
    # We'll batch the updates for efficiency
    count = 0
    for row in items.rows:
        update_data = {}
        
        # Empty text columns
        for col in columns_to_empty:
            if row.get(col):
                update_data[col] = ""
        
        # Truncate embeddings to 768 float32 entries (768 * 4 bytes)
        for col in ["embedding", "full_embedding"]:
            blob = row.get(col)
            if blob and isinstance(blob, bytes):
                target_len = 768 * 4
                if len(blob) > target_len:
                    update_data[col] = blob[:target_len]
        
        if update_data:
            items.update(row["identifier"], update_data)
            count += 1
            if count % 1000 == 0:
                logger.debug(f"Processed {count} rows...")

    logger.info(f"Finished processing {count} rows.")
    
    # VACUUM to reclaim space
    logger.info("Vacuuming database...")
    database.vacuum()
    
    final_size = os.path.getsize(output_db_path) / (1024 * 1024)
    logger.info(f"Cleanup complete. Final size: {final_size:.2f} MB")

if __name__ == "__main__":
    import argparse
    parser = argparse.ArgumentParser(description="Clean up transcript explorer database for sharing.")
    parser.add_argument("input", help="Path to the original summaries.db")
    parser.add_argument("output", help="Path to the cleaned summaries_clean.db")
    
    args = parser.parse_args()
    cleanup_database(args.input, args.output)
