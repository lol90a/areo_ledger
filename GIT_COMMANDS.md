# Git Commands for GitHub Upload

## Step 1: Initialize Git Repository

```bash
# Navigate to project directory
cd e:\areo_ledger

# Initialize git (if not already done)
git init

# Check status
git status
```

## Step 2: Stage All Files

```bash
# Add all files (respects .gitignore)
git add .

# Verify what will be committed
git status
```

## Step 3: Create Initial Commit

```bash
# Commit with descriptive message
git commit -m "Initial commit: AeroLedger crypto aviation platform

- Rust backend with Actix-web and PostgreSQL
- React frontend with TypeScript and Tailwind
- JWT authentication with bcrypt password hashing
- Multi-crypto payment support (BTC, ETH, USDT, USDC, SOL, BNB)
- Admin dashboard with real-time statistics
- Rate limiting and security features
- Comprehensive documentation"
```

## Step 4: Create GitHub Repository

1. Go to https://github.com/new
2. Repository name: `aeroledger` (or `areo_ledger`)
3. Description: `Luxury flight booking and private jet sales platform with cryptocurrency payments`
4. Choose Public or Private
5. **DO NOT** initialize with README, .gitignore, or license (we have them)
6. Click "Create repository"

## Step 5: Connect to GitHub

```bash
# Add remote (replace YOUR_USERNAME with your GitHub username)
git remote add origin https://github.com/YOUR_USERNAME/aeroledger.git

# Verify remote
git remote -v
```

## Step 6: Push to GitHub

```bash
# Rename branch to main (if needed)
git branch -M main

# Push to GitHub
git push -u origin main
```

## Step 7: Verify Upload

1. Go to your GitHub repository
2. Check all files are uploaded
3. Verify README displays correctly
4. Check that .env is NOT uploaded (should be in .gitignore)

## Step 8: Configure Repository

### Add Topics
Go to repository settings and add:
- `rust`
- `react`
- `cryptocurrency`
- `blockchain`
- `web3`
- `actix-web`
- `postgresql`
- `typescript`
- `tailwindcss`
- `jwt`

### Update Repository Details
- Add description
- Add website URL (if deployed)
- Enable Issues
- Enable Discussions (optional)

## Step 9: Create Release (Optional)

```bash
# Tag the release
git tag -a v1.0.0 -m "Initial release: AeroLedger v1.0.0"

# Push tags
git push origin v1.0.0
```

Then create a release on GitHub:
1. Go to Releases
2. Click "Create a new release"
3. Choose tag v1.0.0
4. Title: "AeroLedger v1.0.0 - Initial Release"
5. Description: List features and changes
6. Publish release

## Common Git Commands

### Check Status
```bash
git status
```

### View Changes
```bash
git diff
```

### View Commit History
```bash
git log --oneline
```

### Create New Branch
```bash
git checkout -b feature/new-feature
```

### Switch Branch
```bash
git checkout main
```

### Pull Latest Changes
```bash
git pull origin main
```

### Push Changes
```bash
git add .
git commit -m "Description of changes"
git push origin main
```

## Troubleshooting

### If you get authentication errors:
1. Use Personal Access Token instead of password
2. Go to GitHub Settings > Developer settings > Personal access tokens
3. Generate new token with `repo` scope
4. Use token as password when pushing

### If you need to change remote URL:
```bash
git remote set-url origin https://github.com/YOUR_USERNAME/aeroledger.git
```

### If you accidentally committed .env:
```bash
# Remove from git but keep local file
git rm --cached .env

# Commit the removal
git commit -m "Remove .env from repository"

# Push changes
git push origin main
```

### If you need to undo last commit:
```bash
# Undo commit but keep changes
git reset --soft HEAD~1

# Undo commit and discard changes
git reset --hard HEAD~1
```

## Best Practices

1. **Never commit sensitive data**
   - .env files
   - API keys
   - Passwords
   - Private keys

2. **Write clear commit messages**
   - Use present tense
   - Be descriptive
   - Reference issues if applicable

3. **Commit frequently**
   - Small, logical commits
   - One feature per commit
   - Test before committing

4. **Use branches**
   - main/master for stable code
   - feature/* for new features
   - bugfix/* for bug fixes
   - hotfix/* for urgent fixes

5. **Pull before push**
   - Always pull latest changes
   - Resolve conflicts locally
   - Test after merging

## GitHub Repository Checklist

- [ ] Repository created on GitHub
- [ ] Code pushed successfully
- [ ] README displays correctly
- [ ] .env NOT in repository
- [ ] Topics added
- [ ] Description added
- [ ] License visible
- [ ] Issues enabled
- [ ] All documentation files present
- [ ] No sensitive data committed

## Next Steps After Upload

1. Share repository link
2. Add collaborators (if any)
3. Set up GitHub Actions (CI/CD)
4. Add project board (optional)
5. Create wiki (optional)
6. Add screenshots to README
7. Deploy application
8. Add deployment badges

---

**Your project is ready for GitHub! 🚀**

Follow these steps and your code will be live on GitHub in minutes.
