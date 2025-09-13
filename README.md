# LolzUp - Auto-bump for humans

## Features

- Dialog-based interactions
- Background task processing
- SQLite database storage
- Modern async Python stack
- Docker containerization

## Installation

1. Clone repository:
   ```bash
   git clone https://github.com/awalki/lolzup.git
   cd lolzup
   ```

2. Create `.env` file:
   ```env
   LOLZ_TOKEN=your_lolz_token
   BOT_TOKEN=your_telegram_bot_token
   ADMIN_ID=your_telegram_id
   ```

3. Run:
   ```bash
   docker-compose up -d
   ```