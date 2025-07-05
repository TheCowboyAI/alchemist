# CIM Production Deployment Checklist

## Pre-Deployment Phase

### Infrastructure Preparation

- [ ] **Hardware Provisioned**
  - [ ] Minimum specs verified (16 cores, 64GB RAM, 512GB SSD)
  - [ ] GPU available for AI workloads (if required)
  - [ ] Network connectivity tested
  - [ ] Storage performance benchmarked

- [ ] **NixOS Base System**
  - [ ] NixOS 24.05 or later installed
  - [ ] Base system hardened per security policy
  - [ ] SSH access configured with key-based auth only
  - [ ] Firewall rules configured
  - [ ] SELinux/AppArmor enabled

- [ ] **NATS Hub Infrastructure**
  - [ ] NATS cluster deployed (minimum 3 nodes)
  - [ ] JetStream enabled and configured
  - [ ] TLS certificates generated and deployed
  - [ ] Monitoring endpoints exposed
  - [ ] Backup strategy implemented

### Security Preparation

- [ ] **Identity Setup (cim-domain-identity)**
  - [ ] Node identity created in Identity Domain
  - [ ] Identity linked to managing organization
  - [ ] Verification level set (minimum: Basic)
  - [ ] Authentication method configured
  - [ ] Identity claims populated

- [ ] **Cryptographic Infrastructure (cim-keys)**
  - [ ] Root CA created and secured
  - [ ] Intermediate CA deployed
  - [ ] Leaf node certificates generated via PKI
  - [ ] Ed25519 signing keys created
  - [ ] TLS certificates for all services
  - [ ] YubiKey provisioned for high-security nodes
  - [ ] SSH keys generated for administration
  - [ ] Certificate rotation schedule defined
  - [ ] Hardware token backup procedures documented

- [ ] **Security Policies (cim-security)**
  - [ ] Node security policies defined
  - [ ] NATS subject permissions configured
  - [ ] Policy rules for allow/deny patterns
  - [ ] Claims-based authorization configured
  - [ ] Audit event policies created

- [ ] **Secrets Management**
  - [ ] sops-nix or Vault configured
  - [ ] Private keys encrypted and stored
  - [ ] NATS credentials secured
  - [ ] API keys encrypted and stored
  - [ ] Database passwords secured
  - [ ] JWT signing keys generated
  - [ ] Backup encryption keys created

- [ ] **Access Control**
  - [ ] User accounts created with appropriate roles
  - [ ] Service accounts configured
  - [ ] Certificate-based authentication enabled
  - [ ] RBAC policies defined
  - [ ] Audit logging enabled
  - [ ] MFA configured for admin access
  - [ ] Security context propagation tested

### Configuration Preparation

- [ ] **Domain Configuration**
  - [ ] Domain objects defined in Alchemist
  - [ ] Graph composition validated
  - [ ] Resource allocations calculated
  - [ ] Dependencies mapped
  - [ ] Configuration exported to Nix

- [ ] **Network Configuration**
  - [ ] Static IPs or DNS configured
  - [ ] Firewall rules defined
  - [ ] Load balancer configured (if applicable)
  - [ ] VPN access configured
  - [ ] Network segmentation implemented

## Deployment Phase

### Initial Deployment

- [ ] **Generate Configuration**
  ```bash
  alchemist deploy generate \
    --instance cim-prod-001 \
    --validate \
    --output ./deploy/
  ```

- [ ] **Review Configuration**
  - [ ] Nix configuration syntax valid
  - [ ] Resource limits appropriate
  - [ ] Security settings correct
  - [ ] Domain configurations complete
  - [ ] Integration points defined

- [ ] **Deploy Base System**
  ```bash
  nixos-anywhere \
    --flake .#cim-prod-001 \
    --target-host root@cim-001.example.com \
    --disk-encryption-keys /secure/keys/disk.key
  ```

- [ ] **Verify Base Deployment**
  - [ ] System boots successfully
  - [ ] All services start
  - [ ] Network connectivity confirmed
  - [ ] Storage mounted correctly
  - [ ] Base monitoring working

### NATS Leaf Node Setup

- [ ] **Configure Leaf Node**
  - [ ] Leaf credentials deployed
  - [ ] TLS certificates in place
  - [ ] Subject mappings configured
  - [ ] JetStream streams created
  - [ ] Consumers configured

- [ ] **Test Connectivity**
  ```bash
  # From CIM instance
  nats --server=localhost:4222 rtt
  nats --server=localhost:4222 pub test "hello"
  nats --server=localhost:4222 sub ">"
  ```

- [ ] **Verify Leaf Status**
  ```bash
  curl http://localhost:8222/leafz | jq
  check-leaf-health
  ```

### Domain Services Deployment

- [ ] **Deploy Core Domains**
  - [ ] Graph domain started
  - [ ] Workflow domain started
  - [ ] Agent domain started
  - [ ] Identity domain started
  - [ ] ConceptualSpaces domain started

- [ ] **Verify Domain Health**
  ```bash
  for domain in graph workflow agent identity; do
    systemctl status cim-$domain
    curl http://localhost:9090/api/v1/domains/$domain/health
  done
  ```

- [ ] **Test Domain Functionality**
  - [ ] Create test graph
  - [ ] Execute test workflow
  - [ ] Deploy test agent
  - [ ] Create test identity
  - [ ] Perform conceptual analysis

### Agent and Service Deployment

- [ ] **Deploy Initial Agents**
  ```typescript
  await alchemist.deployAgent(workflowAgent, "cim-prod-001");
  await alchemist.deployAgent(analysisAgent, "cim-prod-001");
  ```

- [ ] **Deploy Custom Services**
  - [ ] Service bundles created
  - [ ] Resources allocated
  - [ ] Dependencies resolved
  - [ ] Health checks defined
  - [ ] Services deployed

- [ ] **Verify Deployments**
  - [ ] Agents responding to commands
  - [ ] Services health checks passing
  - [ ] Event flow working
  - [ ] Resource usage acceptable
  - [ ] Performance metrics normal

## Post-Deployment Phase

### Monitoring Setup

- [ ] **Prometheus Configuration**
  - [ ] Scrapers configured for all services
  - [ ] Recording rules defined
  - [ ] Alerting rules created
  - [ ] Long-term storage configured
  - [ ] Federation setup (if applicable)

- [ ] **Grafana Dashboards**
  - [ ] CIM overview dashboard imported
  - [ ] Domain-specific dashboards configured
  - [ ] NATS leaf node dashboard setup
  - [ ] Alert panels configured
  - [ ] User access configured

- [ ] **Alerting Configuration**
  - [ ] Alert channels configured (email, Slack, PagerDuty)
  - [ ] Escalation policies defined
  - [ ] Runbook links added
  - [ ] Test alerts sent
  - [ ] On-call schedule configured

### Performance Validation

- [ ] **Load Testing**
  ```bash
  cim-load-test \
    --target cim-prod-001 \
    --events-per-sec 10000 \
    --duration 30m \
    --report load-test-results.json
  ```

- [ ] **Performance Benchmarks**
  - [ ] Event throughput > 100k/sec
  - [ ] P99 latency < 10ms
  - [ ] Memory usage stable
  - [ ] CPU usage < 70%
  - [ ] No message loss

- [ ] **Stress Testing**
  - [ ] Burst load handled
  - [ ] Recovery from overload
  - [ ] Circuit breakers working
  - [ ] Rate limiting effective
  - [ ] Graceful degradation verified

### Security Validation

- [ ] **Identity Verification**
  ```bash
  # Verify node identity
  alchemist identity verify --node cim-prod-001
  
  # Check verification level
  alchemist identity status --node cim-prod-001
  
  # List identity claims
  alchemist identity claims --node cim-prod-001
  ```

- [ ] **Certificate Validation**
  ```bash
  # Verify certificate chain
  openssl verify -CAfile ca-chain.crt node.crt
  
  # Check certificate details
  openssl x509 -in node.crt -text -noout
  
  # TLS verification
  testssl.sh cim-001.example.com:4222
  
  # NATS TLS test
  nats --tlscert=node.crt --tlskey=node.key --tlsca=ca.crt rtt
  ```

- [ ] **Security Policy Testing**
  ```bash
  # Test policy evaluation
  alchemist security test-policy --node cim-prod-001 --action publish --resource "test.subject"
  
  # Verify audit logging
  alchemist security audit-log --node cim-prod-001 --last 1h
  
  # Check policy violations
  alchemist security violations --node cim-prod-001
  ```

- [ ] **Access Control Testing**
  - [ ] Certificate authentication working
  - [ ] Unauthorized access blocked
  - [ ] Role permissions enforced
  - [ ] Policy rules evaluated correctly
  - [ ] Claims-based auth functioning
  - [ ] API authentication required
  - [ ] Audit logs capturing all events
  - [ ] Encryption verified end-to-end

- [ ] **Key Management Verification**
  - [ ] All keys properly stored
  - [ ] Key rotation tested
  - [ ] Hardware tokens accessible
  - [ ] Backup keys recoverable
  - [ ] Key usage audited

- [ ] **Security Scan**
  ```bash
  # Vulnerability scanning
  trivy image cim-prod-001
  
  # Network scan
  nmap -sV -p- cim-001.example.com
  
  # Check for exposed secrets
  trufflehog filesystem /etc/nixos/
  ```

- [ ] **Penetration Testing**
  - [ ] External pen test scheduled
  - [ ] Certificate validation tested
  - [ ] Policy bypass attempts failed
  - [ ] Results reviewed
  - [ ] Critical issues resolved
  - [ ] Medium issues tracked
  - [ ] Retest completed

### Operational Readiness

- [ ] **Documentation**
  - [ ] Runbooks completed
  - [ ] Architecture diagrams updated
  - [ ] API documentation published
  - [ ] Troubleshooting guide ready
  - [ ] Contact list current

- [ ] **Backup and Recovery**
  - [ ] Backup jobs scheduled
  - [ ] Backup verification passing
  - [ ] Recovery procedure tested
  - [ ] RTO/RPO validated
  - [ ] Off-site backups configured

- [ ] **Disaster Recovery**
  - [ ] DR site configured
  - [ ] Failover tested
  - [ ] Data replication verified
  - [ ] Communication plan ready
  - [ ] Recovery drills scheduled

## Go-Live Phase

### Final Checks

- [ ] **System Health**
  ```bash
  alchemist health-check --instance cim-prod-001 --comprehensive
  ```

- [ ] **Integration Tests**
  ```bash
  alchemist test integration --target cim-prod-001 --full-suite
  ```

- [ ] **Rollback Plan**
  - [ ] Rollback procedure documented
  - [ ] Previous version archived
  - [ ] Database backups taken
  - [ ] Team briefed on procedure
  - [ ] Rollback tested in staging

### Cutover

- [ ] **Communication**
  - [ ] Stakeholders notified
  - [ ] Maintenance window scheduled
  - [ ] Status page updated
  - [ ] Support team ready
  - [ ] Escalation paths confirmed

- [ ] **Traffic Migration**
  - [ ] DNS updated (if applicable)
  - [ ] Load balancer configured
  - [ ] Traffic gradually shifted
  - [ ] Metrics monitored
  - [ ] Rollback ready

- [ ] **Validation**
  - [ ] End-to-end tests passing
  - [ ] User acceptance verified
  - [ ] Performance acceptable
  - [ ] No critical errors
  - [ ] Business metrics normal

## Post Go-Live

### Stabilization (First 24 Hours)

- [ ] **Active Monitoring**
  - [ ] 24/7 coverage arranged
  - [ ] Dashboards monitored
  - [ ] Alerts reviewed
  - [ ] Performance tracked
  - [ ] Issues logged

- [ ] **Issue Resolution**
  - [ ] P1 issues addressed immediately
  - [ ] Root cause analysis started
  - [ ] Workarounds implemented
  - [ ] Communication maintained
  - [ ] Fixes deployed

### Optimization (First Week)

- [ ] **Performance Tuning**
  - [ ] Bottlenecks identified
  - [ ] Cache settings optimized
  - [ ] Query performance improved
  - [ ] Resource allocation adjusted
  - [ ] Autoscaling configured

- [ ] **Process Improvement**
  - [ ] Deployment process reviewed
  - [ ] Automation opportunities identified
  - [ ] Documentation updated
  - [ ] Training needs assessed
  - [ ] Feedback collected

### Handover (First Month)

- [ ] **Knowledge Transfer**
  - [ ] Operations team trained
  - [ ] Support procedures documented
  - [ ] Common issues cataloged
  - [ ] Escalation tested
  - [ ] Handover signed off

- [ ] **Continuous Improvement**
  - [ ] Metrics baseline established
  - [ ] SLO targets set
  - [ ] Improvement backlog created
  - [ ] Regular reviews scheduled
  - [ ] Success metrics defined

## Sign-Off

### Technical Sign-Off

- [ ] **Infrastructure Lead**: ___________________ Date: ___________
- [ ] **Security Lead**: ___________________ Date: ___________
- [ ] **Operations Lead**: ___________________ Date: ___________
- [ ] **Development Lead**: ___________________ Date: ___________

### Business Sign-Off

- [ ] **Product Owner**: ___________________ Date: ___________
- [ ] **Business Stakeholder**: ___________________ Date: ___________
- [ ] **Executive Sponsor**: ___________________ Date: ___________

## Appendix: Emergency Contacts

| Role               | Name | Phone | Email | Escalation |
| ------------------ | ---- | ----- | ----- | ---------- |
| Incident Commander |      |       |       | Primary    |
| Technical Lead     |      |       |       | Primary    |
| Infrastructure     |      |       |       | Secondary  |
| Security           |      |       |       | Secondary  |
| Business Owner     |      |       |       | Tertiary   |

## Appendix: Critical Commands

```bash
# Emergency shutdown
systemctl stop cim-* nats

# Emergency restart
systemctl restart nats && systemctl restart cim-*

# Disable problematic domain
systemctl disable --now cim-<domain>

# Force leaf reconnect
systemctl restart nats

# Clear JetStream
nats stream purge DOMAIN_EVENTS --force

# Enable debug logging
echo "debug: true" >> /etc/nats/nats.conf && systemctl reload nats
```

---

This checklist ensures a systematic and thorough deployment of CIM instances in production environments. 