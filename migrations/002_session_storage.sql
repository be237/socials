-- Socials Database Schema v0.2
-- Add session storage for user connections

-- Add session_data column to accounts table
ALTER TABLE accounts ADD COLUMN session_data TEXT;
