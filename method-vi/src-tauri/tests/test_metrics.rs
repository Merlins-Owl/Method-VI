use method_vi_lib::agents::governance_telemetry::{GovernanceTelemetryAgent, MetricStatus};
use std::env;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    env_logger::init();

    println!("=== Method-VI Governance & Telemetry Agent Test ===\n");

    // Get API key from environment
    let api_key = env::var("ANTHROPIC_API_KEY").expect(
        "ANTHROPIC_API_KEY environment variable must be set. \
        Run: set ANTHROPIC_API_KEY=your-key-here",
    );

    // Create agent
    println!("1. Initializing Governance & Telemetry Agent...");
    let mut agent = GovernanceTelemetryAgent::new(api_key)?;
    println!("   ✓ Agent initialized\n");

    // Sample baseline content for E_baseline calculation
    let baseline_content = r#"
# Project Charter: E-Commerce Platform Modernization

## Objectives
1. Migrate legacy monolithic application to microservices architecture
2. Improve system performance by 50%
3. Enhance user experience with modern UI/UX
4. Implement comprehensive monitoring and observability
5. Reduce deployment time from weeks to hours

## Scope
- User authentication and authorization service
- Product catalog service
- Shopping cart and checkout service
- Order management service
- Payment processing integration
- Frontend SPA using React

## Success Criteria
- All services deployed independently
- 99.9% uptime SLA maintained
- Page load times under 2 seconds
- Zero data loss during migration
- Full test coverage (>80%)

## Out of Scope
- Mobile native applications
- International payment methods (Phase 2)
- Advanced analytics dashboard (Phase 2)
"#;

    // Calculate and lock E_baseline
    println!("2. Calculating E_baseline from baseline content...");
    let e_baseline = agent.calculate_e_baseline(baseline_content, 1)?;
    println!("   E_baseline: {} words", e_baseline);

    agent.lock_e_baseline(1)?;
    println!("   ✓ E_baseline locked at Step 1\n");

    // Sample content for metric calculation (Step 3 output)
    let step_content = r#"
# Diagnostic Summary: E-Commerce Platform Analysis

## System Architecture Assessment

### Current State
The legacy monolithic application consists of a single Java Spring application
deployed as a WAR file. The architecture exhibits several structural issues:

1. **Tight Coupling**: All business logic is intertwined within the same codebase,
   making independent deployment impossible.

2. **Database Bottlenecks**: Single PostgreSQL instance handles all data operations,
   creating performance constraints.

3. **Scalability Limitations**: Horizontal scaling requires duplicating the entire
   application stack, leading to resource inefficiency.

### Proposed Microservices Architecture

We will decompose the monolith into six independent services:

1. **Authentication Service** - Handles user login, session management, JWT tokens
2. **Product Catalog Service** - Manages product data, search, and recommendations
3. **Cart Service** - Shopping cart operations with Redis caching
4. **Order Service** - Order processing, fulfillment tracking
5. **Payment Service** - Integration with Stripe and PayPal
6. **API Gateway** - Request routing, rate limiting, authentication

### Technical Approach

Each service will be:
- Containerized using Docker
- Deployed to Kubernetes cluster
- Backed by independent database instances
- Monitored via Prometheus and Grafana
- Logged to centralized ELK stack

### Migration Strategy

Phase 1 (Weeks 1-4):
- Set up Kubernetes infrastructure
- Migrate authentication service
- Implement API gateway

Phase 2 (Weeks 5-8):
- Migrate product catalog
- Migrate cart service
- Deploy caching layer

Phase 3 (Weeks 9-12):
- Migrate order and payment services
- Cutover production traffic
- Decommission monolith

### Performance Improvements

Expected outcomes:
- API response times: <200ms (currently 800ms)
- Database query performance: 60% improvement
- System throughput: 3x increase
- Deployment frequency: Daily releases vs monthly

### Risk Mitigation

1. **Data Consistency**: Implement distributed transactions using Saga pattern
2. **Service Discovery**: Use Consul for dynamic service registration
3. **Circuit Breaking**: Hystrix for fault tolerance
4. **Testing**: Comprehensive integration tests before migration

## Alignment with Charter

This diagnostic directly addresses Charter objectives:
- ✓ Microservices architecture (Objective 1)
- ✓ Performance improvements projected at 62% (exceeds 50% target)
- ✓ Modern deployment practices enable hourly releases
- ✓ Monitoring via Prometheus/Grafana (Objective 4)

All proposed work remains within defined scope boundaries.
"#;

    // Charter objectives for IAS calculation
    let charter_objectives = r#"
1. Migrate legacy monolithic application to microservices architecture
2. Improve system performance by 50%
3. Enhance user experience with modern UI/UX
4. Implement comprehensive monitoring and observability
5. Reduce deployment time from weeks to hours
"#;

    // Calculate all metrics
    println!("3. Calculating Critical 6 Metrics for Step 3...\n");
    let metrics = agent
        .calculate_metrics(step_content, charter_objectives, 3)
        .await?;

    // Print detailed results for each metric
    println!("{}", "=".repeat(80));
    println!("METRIC RESULTS");
    println!("{}", "=".repeat(80));

    // CI - Coherence Index
    if let Some(ref ci) = metrics.ci {
        println!("\n📊 CI - Coherence Index");
        println!("   Value: {:.2}", ci.value);
        println!("   Status: {:?}", ci.status);
        println!("   Threshold: Pass ≥{:.2}, Warning ≥{:.2}, Fail <{:.2}",
                 ci.threshold.pass,
                 ci.threshold.warning.unwrap_or(0.0),
                 ci.threshold.halt.unwrap_or(0.0));
        println!("\n   Inputs Used:");
        for input in &ci.inputs_used {
            println!("      • {} = {:?} (from {})", input.name, input.value, input.source);
        }
        println!("\n   Calculation Method:");
        println!("      {}", ci.calculation_method);
        println!("\n   Interpretation:");
        println!("      {}", ci.interpretation);
        if let Some(ref rec) = ci.recommendation {
            println!("\n   ⚠️  Recommendation:");
            println!("      {}", rec);
        }

        // Verify required fields
        assert!(ci.value >= 0.0 && ci.value <= 1.0, "CI value out of range");
        assert!(!ci.inputs_used.is_empty(), "CI missing inputs");
        assert!(!ci.calculation_method.is_empty(), "CI missing calculation method");
        assert!(!ci.interpretation.is_empty(), "CI missing interpretation");
    }

    // EV - Expansion Variance
    if let Some(ref ev) = metrics.ev {
        println!("\n📊 EV - Expansion Variance");
        println!("   Value: {:.1}%", ev.value);
        println!("   Status: {:?}", ev.status);
        println!("   Threshold: Pass ≤{:.0}%, Warning ≤{:.0}%, Fail >{:.0}%",
                 ev.threshold.pass,
                 ev.threshold.warning.unwrap_or(0.0),
                 ev.threshold.halt.unwrap_or(0.0));
        println!("\n   Inputs Used:");
        for input in &ev.inputs_used {
            println!("      • {} = {:?} (from {})", input.name, input.value, input.source);
        }
        println!("\n   Calculation Method:");
        println!("      {}", ev.calculation_method);
        println!("\n   Interpretation:");
        println!("      {}", ev.interpretation);
        if let Some(ref rec) = ev.recommendation {
            println!("\n   ⚠️  Recommendation:");
            println!("      {}", rec);
        }

        // Verify required fields
        assert!(ev.value >= 0.0, "EV value cannot be negative");
        assert!(ev.inputs_used.len() == 2, "EV should have 2 inputs (E_current, E_baseline)");
        assert!(!ev.calculation_method.is_empty(), "EV missing calculation method");
        assert!(!ev.interpretation.is_empty(), "EV missing interpretation");
    }

    // IAS - Intent Alignment Score
    if let Some(ref ias) = metrics.ias {
        println!("\n📊 IAS - Intent Alignment Score");
        println!("   Value: {:.2}", ias.value);
        println!("   Status: {:?}", ias.status);
        println!("   Threshold: Pass ≥{:.2}, Warning ≥{:.2}, Fail <{:.2}",
                 ias.threshold.pass,
                 ias.threshold.warning.unwrap_or(0.0),
                 ias.threshold.halt.unwrap_or(0.0));
        println!("\n   Inputs Used:");
        for input in &ias.inputs_used {
            println!("      • {} (from {})", input.name, input.source);
        }
        println!("\n   Calculation Method:");
        println!("      {}", ias.calculation_method);
        println!("\n   Interpretation:");
        println!("      {}", ias.interpretation);
        if let Some(ref rec) = ias.recommendation {
            println!("\n   ⚠️  Recommendation:");
            println!("      {}", rec);
        }

        // Verify required fields
        assert!(ias.value >= 0.0 && ias.value <= 1.0, "IAS value out of range");
        assert!(!ias.inputs_used.is_empty(), "IAS missing inputs");
        assert!(!ias.calculation_method.is_empty(), "IAS missing calculation method");
        assert!(!ias.interpretation.is_empty(), "IAS missing interpretation");
    }

    // EFI - Execution Fidelity Index
    if let Some(ref efi) = metrics.efi {
        println!("\n📊 EFI - Execution Fidelity Index");
        println!("   Value: {:.1}%", efi.value);
        println!("   Status: {:?}", efi.status);
        println!("   Threshold: Pass ≥{:.0}%, Warning ≥{:.0}%, Fail <{:.0}%",
                 efi.threshold.pass,
                 efi.threshold.warning.unwrap_or(0.0),
                 efi.threshold.halt.unwrap_or(0.0));
        println!("\n   Inputs Used:");
        for input in &efi.inputs_used {
            println!("      • {} = {:?} (from {})", input.name, input.value, input.source);
        }
        println!("\n   Calculation Method:");
        println!("      {}", efi.calculation_method);
        println!("\n   Interpretation:");
        println!("      {}", efi.interpretation);
        if let Some(ref rec) = efi.recommendation {
            println!("\n   ⚠️  Recommendation:");
            println!("      {}", rec);
        }

        // Verify required fields
        assert!(efi.value >= 0.0 && efi.value <= 100.0, "EFI value out of range");
        assert!(efi.inputs_used.len() >= 2, "EFI should have at least 2 inputs");
        assert!(!efi.calculation_method.is_empty(), "EFI missing calculation method");
        assert!(!efi.interpretation.is_empty(), "EFI missing interpretation");
    }

    // SEC - Scope Expansion Count
    if let Some(ref sec) = metrics.sec {
        println!("\n📊 SEC - Scope Expansion Count");
        println!("   Value: {:.0}%", sec.value);
        println!("   Status: {:?}", sec.status);
        println!("   Threshold: Pass ={:.0}%",
                 sec.threshold.pass);
        println!("\n   Inputs Used:");
        for input in &sec.inputs_used {
            println!("      • {} = {:?} (from {})", input.name, input.value, input.source);
        }
        println!("\n   Calculation Method:");
        println!("      {}", sec.calculation_method);
        println!("\n   Interpretation:");
        println!("      {}", sec.interpretation);
        if let Some(ref rec) = sec.recommendation {
            println!("\n   ⚠️  Recommendation:");
            println!("      {}", rec);
        }

        // Verify required fields
        assert!(sec.value >= 0.0 && sec.value <= 100.0, "SEC value out of range");
        assert!(!sec.inputs_used.is_empty(), "SEC missing inputs");
        assert!(!sec.calculation_method.is_empty(), "SEC missing calculation method");
        assert!(!sec.interpretation.is_empty(), "SEC missing interpretation");
    }

    // PCI - Process Compliance Index
    if let Some(ref pci) = metrics.pci {
        println!("\n📊 PCI - Process Compliance Index");
        println!("   Value: {:.2}", pci.value);
        println!("   Status: {:?}", pci.status);
        println!("   Threshold: Pass ≥{:.2}, Warning ≥{:.2}, Fail <{:.2}",
                 pci.threshold.pass,
                 pci.threshold.warning.unwrap_or(0.0),
                 pci.threshold.halt.unwrap_or(0.0));
        println!("\n   Inputs Used:");
        for input in &pci.inputs_used {
            println!("      • {} = {:?} (from {})", input.name, input.value, input.source);
        }
        println!("\n   Calculation Method:");
        println!("      {}", pci.calculation_method);
        println!("\n   Interpretation:");
        println!("      {}", pci.interpretation);
        if let Some(ref rec) = pci.recommendation {
            println!("\n   ⚠️  Recommendation:");
            println!("      {}", rec);
        }

        // Verify required fields
        assert!(pci.value >= 0.0 && pci.value <= 1.0, "PCI value out of range");
        assert!(!pci.inputs_used.is_empty(), "PCI missing inputs");
        assert!(!pci.calculation_method.is_empty(), "PCI missing calculation method");
        assert!(!pci.interpretation.is_empty(), "PCI missing interpretation");
    }

    // Summary
    println!("\n{}", "=".repeat(80));
    println!("SUMMARY");
    println!("{}", "=".repeat(80));

    let mut pass_count = 0;
    let mut warning_count = 0;
    let mut fail_count = 0;

    for metric_result in [&metrics.ci, &metrics.ev, &metrics.ias, &metrics.efi, &metrics.sec, &metrics.pci].iter() {
        if let Some(m) = metric_result {
            match m.status {
                MetricStatus::Pass => pass_count += 1,
                MetricStatus::Warning => warning_count += 1,
                MetricStatus::Fail => fail_count += 1,
            }
        }
    }

    println!("\n   ✓ Pass: {}", pass_count);
    println!("   ⚠ Warning: {}", warning_count);
    println!("   ✕ Fail: {}", fail_count);

    // Check HALT/PAUSE conditions
    println!("\n4. Checking HALT/PAUSE conditions...");
    let test_step = 6; // Use Step 6 to test all metrics including EFI
    if let Some(halt_reason) = agent.check_halt_conditions(&metrics, test_step) {
        println!("   🛑 HALT TRIGGERED: {}", halt_reason);
    } else if let Some(pause_reason) = agent.check_pause_conditions(&metrics) {
        println!("   ⏸️  PAUSE RECOMMENDED: {}", pause_reason);
    } else {
        println!("   ✓ All metrics within acceptable thresholds");
    }

    println!("\n{}", "=".repeat(80));
    println!("✅ TEST COMPLETE - All metrics calculated successfully!");
    println!("{}", "=".repeat(80));

    Ok(())
}
