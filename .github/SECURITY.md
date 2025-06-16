# Security Policy

## Our Commitment to Security

The Information Alchemist project takes security seriously. We appreciate the security research community's efforts to responsibly disclose vulnerabilities, and we are committed to working with researchers and users to address security issues promptly and transparently.

## Supported Versions

We provide security updates for the following versions of Information Alchemist:

| Version          | Supported                                 |
| ---------------- | ----------------------------------------- |
| main             | ✅ Always supported                        |
| Latest release   | ✅ Supported                               |
| Previous release | ✅ Supported for 90 days after new release |
| Older releases   | ❌ Not supported                           |

## Security Scope

### In Scope
The following components and areas are within our security scope:

- **Core Application**: Main Alchemist application code
- **Domain Modules**: All `cim-*` domain modules and their interactions
- **NATS Integration**: Message handling, authentication, and authorization
- **Event Sourcing**: Event store security and data integrity
- **Authentication & Authorization**: User and system authentication mechanisms
- **Data Processing**: Input validation and sanitization
- **Configuration**: Security-related configuration handling
- **Dependencies**: Third-party crates and their security implications

### Out of Scope
The following are generally outside our security scope:

- **Infrastructure**: Deployment environments, operating systems, network security
- **Third-party Services**: External services that integrate with Alchemist
- **Physical Security**: Physical access to systems running Alchemist
- **Social Engineering**: Attacks targeting users or administrators
- **Denial of Service**: Resource exhaustion attacks (unless they reveal other vulnerabilities)

## Reporting Security Vulnerabilities

### How to Report

**Please do NOT create public GitHub issues for security vulnerabilities.**

Instead, please report security vulnerabilities through one of these channels:

1. **Email**: Send details to security@cowboy.ai
2. **GitHub Security Advisory**: Use GitHub's private vulnerability reporting feature
3. **GPG Encrypted Email**: Use our public GPG key for sensitive information

### Required Information

When reporting a security vulnerability, please include:

- **Summary**: A brief description of the vulnerability
- **Impact**: What type of access or information could be exposed
- **Steps to Reproduce**: Detailed steps to reproduce the vulnerability
- **Proof of Concept**: Code, configuration, or commands demonstrating the issue
- **Environment**: Version, operating system, and relevant configuration details
- **Suggested Fix**: If you have ideas for how to fix the issue

### Example Report Template

```
Subject: [SECURITY] Brief description of vulnerability

SUMMARY:
Brief description of the vulnerability and its potential impact.

AFFECTED COMPONENTS:
- Component 1 (version X.Y.Z)
- Component 2 (commit hash ABC123)

VULNERABILITY DETAILS:
Detailed technical description of the vulnerability.

REPRODUCTION STEPS:
1. Step one
2. Step two
3. Step three

IMPACT:
Description of what an attacker could accomplish.

PROOF OF CONCEPT:
[Code, commands, or configuration demonstrating the issue]

SUGGESTED MITIGATION:
[If you have ideas for fixes]

DISCOVERER:
[Your name/handle and affiliation, if you want to be credited]
```

## Response Process

### Timeline
We strive to respond to security reports according to the following timeline:

- **Initial Response**: Within 24-48 hours
- **Confirmation**: Within 72 hours  
- **Status Updates**: Every 7 days until resolution
- **Resolution**: Varies by severity and complexity

### Severity Classification

We classify security vulnerabilities using the following severity levels:

#### Critical (CVSS 9.0-10.0)
- Remote code execution
- Privilege escalation to system administrator
- Complete data exfiltration

#### High (CVSS 7.0-8.9)
- Significant data exposure
- Authentication bypass
- Privilege escalation within application

#### Medium (CVSS 4.0-6.9)
- Limited data exposure
- Denial of service
- Information disclosure

#### Low (CVSS 0.1-3.9)
- Minor information disclosure
- Configuration issues with minimal impact

### Response Actions

1. **Acknowledgment**: We acknowledge receipt of your report
2. **Investigation**: We investigate and reproduce the vulnerability
3. **Confirmation**: We confirm the vulnerability and assess its impact
4. **Development**: We develop and test a fix
5. **Disclosure**: We coordinate disclosure timing with the reporter
6. **Release**: We release the fix and publish a security advisory

## Coordinated Disclosure

We follow a coordinated disclosure process:

- **90-day disclosure deadline**: We aim to fix vulnerabilities within 90 days
- **Public disclosure**: After a fix is available or 90 days have passed
- **Credit**: We provide credit to security researchers (if desired)
- **CVE assignment**: We work with CVE authorities when appropriate

## Security Best Practices

### For Users
- **Keep Updated**: Always use the latest version of Information Alchemist
- **Secure Configuration**: Follow security configuration guidelines
- **Access Control**: Implement proper access controls for your deployment
- **Monitoring**: Monitor for unusual activity in your logs
- **Network Security**: Use TLS/SSL for all network communications

### For Developers
- **Input Validation**: Validate all inputs from external sources
- **Authentication**: Implement proper authentication and authorization
- **Secrets Management**: Never commit secrets to version control
- **Dependencies**: Keep dependencies updated and audit for vulnerabilities
- **Code Review**: All security-related code must be reviewed
- **Testing**: Include security testing in your development process

## Security Architecture

### Threat Model
Information Alchemist's threat model considers:

- **Malicious Input**: Crafted data designed to exploit parsing or processing vulnerabilities
- **Network Attacks**: Man-in-the-middle, replay attacks, and protocol exploits
- **Privilege Escalation**: Attempts to gain unauthorized access to system resources
- **Data Exfiltration**: Unauthorized access to sensitive graph data or events
- **Service Disruption**: Attacks designed to make the system unavailable

### Security Controls
Our security architecture includes:

- **Input Sanitization**: All external inputs are validated and sanitized
- **Authentication**: Strong authentication mechanisms for NATS and system access
- **Authorization**: Role-based access control for different system functions
- **Encryption**: Data in transit is encrypted using TLS 1.3
- **Event Integrity**: Cryptographic verification of event chains
- **Audit Logging**: Comprehensive logging of security-relevant events

## Security Testing

We conduct regular security testing including:

- **Static Analysis**: Automated code analysis for security vulnerabilities
- **Dependency Scanning**: Regular scans of third-party dependencies
- **Integration Testing**: Security testing of component interactions
- **Penetration Testing**: Periodic external security assessments

## Incident Response

In the event of a security incident:

1. **Immediate Response**: Contain the incident and assess impact
2. **Investigation**: Determine root cause and scope
3. **Communication**: Notify affected users and stakeholders
4. **Recovery**: Implement fixes and restore normal operations
5. **Post-Incident**: Conduct post-incident review and improve processes

## Contact Information

- **Security Team**: security@cowboy.ai
- **GPG Key**: [Link to public GPG key]
- **General Contact**: info@cowboy.ai

## Recognition

We maintain a security researcher hall of fame to recognize those who help keep Information Alchemist secure. Researchers who responsibly disclose vulnerabilities may be eligible for:

- Public recognition (if desired)
- Listing in our security acknowledgments
- Swag and merchandise (for significant findings)

## Legal Safe Harbor

We support safe harbor for security researchers who:

- Make good faith efforts to avoid privacy violations and data destruction
- Report vulnerabilities promptly and allow reasonable time for fixes
- Avoid accessing data beyond what is necessary to demonstrate the vulnerability
- Do not perform attacks that degrade or disrupt services

We will not pursue legal action against researchers who follow these guidelines.

---

Thank you for helping keep Information Alchemist and our community safe!

**Last Updated**: January 12, 2025  
**Version**: 1.0 