# Contributing to OpenRDX

Thank you for your interest in contributing to OpenRDX! This document provides guidelines and instructions for contributing to the project.

## Code of Conduct

By participating in this project, you agree to abide by our Code of Conduct. Please read it before contributing.

## How to Contribute

### 1. Fork and Clone

1. Fork the repository on GitHub
2. Clone your fork:
   ```bash
   git clone https://github.com/your-username/OpenRDX.git
   cd OpenRDX
   ```

### 2. Set Up Development Environment

1. Install prerequisites:
   - Docker and Docker Compose
   - Rust toolchain (for core service)
   - Python 3.8+ (for backend)
   - Node.js 16+ (for frontend)
   - mkcert (for SSL)

2. Set up SSL certificates:
   ```bash
   ./scripts/generate-ssl.sh
   ```

3. Start the development environment:
   ```bash
   docker-compose up -d
   ```

### 3. Create a Branch

Create a new branch for your feature or bugfix:
```bash
git checkout -b feature/your-feature-name
# or
git checkout -b fix/your-bug-fix
```

### 4. Development Guidelines

#### Core Service (Rust)
- Follow Rust style guidelines
- Write unit tests for new functionality
- Run tests: `cargo test`
- Check formatting: `cargo fmt`
- Run linter: `cargo clippy`

#### Backend Service (Django)
- Follow PEP 8 style guide
- Write tests for new features
- Run tests: `python manage.py test`
- Check formatting: `black .`
- Run linter: `flake8`

#### Frontend Service (Angular)
- Follow Angular style guide
- Write unit tests for components
- Run tests: `ng test`
- Check formatting: `ng lint`
- Build check: `ng build --prod`

### 5. Commit Your Changes

1. Stage your changes:
   ```bash
   git add .
   ```

2. Commit with a descriptive message:
   ```bash
   git commit -m "Description of changes"
   ```

Commit message format:
```
<type>(<scope>): <description>

[optional body]

[optional footer]
```

Types:
- feat: New feature
- fix: Bug fix
- docs: Documentation changes
- style: Code style changes
- refactor: Code refactoring
- test: Adding or fixing tests
- chore: Maintenance tasks

### 6. Push and Create Pull Request

1. Push your branch:
   ```bash
   git push origin feature/your-feature-name
   ```

2. Create a Pull Request on GitHub:
   - Use the PR template
   - Describe your changes
   - Link related issues
   - Request review from maintainers

### 7. PR Review Process

1. All PRs require at least one review
2. CI checks must pass
3. Code coverage should not decrease
4. Address review comments
5. Keep PRs focused and small

## Development Workflow

1. Keep your fork up to date:
   ```bash
   git remote add upstream https://github.com/original-owner/OpenRDX.git
   git fetch upstream
   git checkout main
   git merge upstream/main
   ```

2. Rebase your feature branch:
   ```bash
   git checkout feature/your-feature-name
   git rebase main
   ```

## Testing

- Write tests for new features
- Ensure all tests pass
- Maintain or improve code coverage
- Test across different environments

## Documentation

- Update README.md if needed
- Add inline documentation
- Update API documentation
- Document configuration changes

## Release Process

1. Version bump
2. Update changelog
3. Create release tag
4. Deploy to staging
5. Deploy to production

## Questions?

Feel free to open an issue for any questions about contributing. 