# Codecov Setup Guide

To enable test coverage badges for your Longhorn Game Engine repository:

## 1. Sign up for Codecov
1. Go to [codecov.io](https://codecov.io)
2. Sign in with your GitHub account
3. Authorize Codecov to access your repositories

## 2. Add your repository
1. Click "Add a repository" in Codecov dashboard
2. Find and select `longhorn-game-engine`
3. Copy the repository upload token

## 3. Add the token to GitHub Secrets
1. Go to your GitHub repository settings
2. Navigate to Settings → Secrets and variables → Actions
3. Click "New repository secret"
4. Name: `CODECOV_TOKEN`
5. Value: Paste the token from Codecov

## 4. Update README badges
Replace `YOUR_USERNAME` in README.md with your actual GitHub username:
- `https://github.com/YOUR_USERNAME/longhorn-game-engine`
- `https://codecov.io/gh/YOUR_USERNAME/longhorn-game-engine`

## 5. Push and verify
1. Push your changes to trigger the workflow
2. Check GitHub Actions to ensure workflows run successfully
3. Visit Codecov dashboard to see coverage reports
4. Badges in README should now display live data

## Troubleshooting
- If coverage upload fails, verify your CODECOV_TOKEN is set correctly
- Ensure all system dependencies are installed in CI
- Check that tests actually run and generate coverage data
- For private repos, you may need a paid Codecov plan