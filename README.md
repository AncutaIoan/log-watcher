
# Actix Web Log Processing API

This project is a web server built with Actix Web for processing log files uploaded via multipart requests. It parses log entries, computes the duration between "START" and "END" events, and classifies job statuses based on the duration.

## Features

- **Upload Log Files**: Accepts log files through a multipart POST request.
- **Parse Log Entries**: Processes logs with timestamps, PID (process ID), status (START/END), and descriptions.
- **Calculate Durations**: Computes the duration between the "START" and "END" events for each job.
- **Classify Job Status**: Classifies the job as "INFO", "WARNING", or "ERROR" based on the duration.
    - **INFO**: Duration less than or equal to 60 seconds.
    - **WARNING**: Duration greater than 60 seconds but less than or equal to 120 seconds.
    - **ERROR**: Duration greater than 120 seconds.

## Requirements

- **Rust**: The application is built using the Rust programming language.
- **Dependencies**:
    - `actix-web`: For the HTTP server.
    - `actix-multipart`: For handling multipart file uploads.
    - `chrono`: For working with timestamps.
    - `futures-util`: For async stream handling.
    - `serde`: For serializing and deserializing data structures.

## Installation

1. Clone the repository:

   ```bash
   git clone https://github.com/yourusername/actix-log-processing-api.git
   cd actix-log-processing-api
   ```

2. Build the project using Cargo:

   ```bash
   cargo build
   ```

3. Run the server:

   ```bash
   cargo run
   ```

   The server will start on `http://127.0.0.1:8080`.

## API Endpoints

### POST `/upload`

Uploads a log file and returns a JSON response with job results.

#### Request

- **Content-Type**: `multipart/form-data`
- The file should contain log entries with the following format:

  ```
  HH:MM:SS, PID, STATUS, DESCRIPTION
  ```

  Example:

  ```
  12:00:00, 1234, START, Job started
  12:02:00, 1234, END, Job finished
  ```

#### Response

A JSON array containing the results for each job, including:

- `pid`: The process ID.
- `description`: The job description.
- `start`: The start timestamp.
- `end`: The end timestamp.
- `duration_secs`: Duration of the job in seconds.
- `level`: Classification of the job based on duration (`INFO`, `WARNING`, `ERROR`).

Example Response:

```json
[
  {
    "pid": "1234",
    "description": "Job finished",
    "start": "12:00:00",
    "end": "12:02:00",
    "duration_secs": 120,
    "level": "ERROR"
  }
]
```

### Error Handling

- **400 Bad Request**: If the uploaded file contains invalid data (e.g., non-UTF-8 characters, incorrect line format).
- **500 Internal Server Error**: If the server encounters an unexpected error.

## Code Overview

### Log Entry Parsing

The `parse_line` function is responsible for parsing each line of the log file. It expects each line to follow this format:

```
HH:MM:SS, PID, STATUS (START/END), DESCRIPTION
```

The function returns a `LogEntry` struct with the parsed data.

### Processing Logic

- The server reads the uploaded file in chunks.
- For each "START" entry, it stores the entry in a hash map using the PID as the key.
- For each "END" entry, it looks up the corresponding "START" entry and calculates the duration between the two timestamps.
- It classifies the job status based on the duration and adds the result to a list.

### Job Classification

- Jobs with durations greater than 120 seconds are classified as "ERROR".
- Jobs with durations between 60 and 120 seconds are classified as "WARNING".
- Jobs with durations less than or equal to 60 seconds are classified as "INFO".

## Testing

To test the API, you can use a tool like `curl` or Postman to upload a file with the log format described above.

Example `curl` command:

```bash
curl -X POST -F "file=@path_to_log_file.log" http://127.0.0.1:8080/upload
```
