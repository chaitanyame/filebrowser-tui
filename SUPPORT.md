# Support

## Getting Help

If you encounter any issues while using File Browser TUI, here are the best ways to get help:

## Documentation

First, check our documentation:

- 📖 [README.md](README.md) - Getting started guide and keyboard shortcuts
- 🐳 [DOCKER.md](DOCKER.md) - Docker-specific help
- 🧪 [TEST_FIXES.md](TEST_FIXES.md) - Testing information
- 📝 [CONTRIBUTING.md](CONTRIBUTING.md) - How to contribute

## Common Issues

### "Terminal looks weird after running"

Make sure your terminal supports UTF-8 and true color. Try:
```bash
export TERM=xterm-256color
```

### "Files are not being deleted/copied"

Check file permissions on Windows:
```powershell
# Run as administrator if needed
# Or check file properties in Windows Explorer
```

### "Cannot see my files"

- Press `.` to toggle hidden files
- Check your mount path if using Docker
- Verify you have permissions to the directory

### "App crashes when opening a file"

Some file types may not have a default application. Check:
- File associations in Windows
- The file is not corrupted
- The file type is supported

### Docker Issues

See [DOCKER.md](DOCKER.md) for:
- Troubleshooting common Docker problems
- Platform-specific notes
- Permission issues

## Reporting Bugs

Before reporting a bug:

1. **Search existing issues** - Check if someone already reported it
2. **Try the latest version** - Your issue may already be fixed
3. **Gather information**:
   - Your OS and version
   - Rust version (`rustc --version`)
   - Terminal type
   - Steps to reproduce
   - Error messages (with `RUST_BACKTRACE=1`)
   - Screenshots if applicable

### Create an Issue

[Report a bug on GitHub](https://github.com/yourusername/filebrowser-tui/issues/new?labels=bug&template=bug_report.md)

## Feature Requests

We welcome feature suggestions! Before submitting:

1. Check the [Roadmap](README.md#roadmap) for planned features
2. Search existing feature requests
3. Consider if it fits the project scope
4. Think about the Windows TUI context

[Suggest a feature on GitHub](https://github.com/yourusername/filebrowser-tui/issues/new?labels=enhancement&template=feature_request.md)

## Security Issues

**Do not report security issues publicly.** Instead:

- Email: security@example.com
- Or use GitHub's private vulnerability reporting

## Community

### GitHub

- **Issues**: [Submit bugs and feature requests](https://github.com/yourusername/filebrowser-tui/issues)
- **Discussions**: [General discussions and questions](https://github.com/yourusername/filebrowser-tui/discussions)
- **Wiki**: [Documentation and guides](https://github.com/yourusername/filebrowser-tui/wiki)

### Real-time Chat

Join our community chat (if available):
- Discord: [Server invite link]
- IRC: #filebrowser-tui on Libera.chat

## Professional Support

For enterprise support, custom development, or consulting, please contact:
- Email: professional@example.com

## Contributing

We welcome contributions! See [CONTRIBUTING.md](CONTRIBUTING.md) for details.

## Donations

If you find this project useful, consider:

- ⭐ Starring it on GitHub
- 🐛 Reporting bugs
- 📖 Improving documentation
- 💻 Submitting pull requests
- ☕ Buying me a coffee: [Sponsor link]

Thank you for your support!
