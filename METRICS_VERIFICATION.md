# Metrics Verification Guide

**Date**: 2025-12-17
**Test Page**: http://localhost:1420/metrics-test

---

## Quick Access

1. **Start the app**: Navigate to http://localhost:1420/
2. **Click "Metrics Test Page"** button (blue button at bottom)
3. **Or navigate directly**: http://localhost:1420/metrics-test

---

## Verification Checklist

### ✅ Requirement 1: Metrics bar shows 6 metrics

**What to check**:
- Look at the **bottom of the page** (footer)
- Should see 6 compact metric cards in a row: `CI`, `EV`, `IAS`, `EFI`, `SEC`, `PCI`
- Each card shows metric name and value

**How to test**:
1. Navigate to test page
2. Look at footer bar
3. Count the metrics (should be 6)

**Expected Result**: Footer shows all 6 metrics horizontally aligned.

---

### ✅ Requirement 2: Color coding works

**What to check**:
- **Green** border = Pass (value meets or exceeds threshold)
- **Yellow** border = Warning (value below pass but above halt)
- **Red** border = Fail (value below halt threshold)

**How to test**:

#### Test CI Color Coding:

1. On test page, click **"Custom CI"** scenario button
2. Use the slider to adjust CI value:
   - Set to **0.85** → Should show **GREEN** border (≥ 0.80 = pass)
   - Set to **0.75** → Should show **YELLOW** border (≥ 0.70 = warning)
   - Set to **0.45** → Should show **RED** border (< 0.50 = fail)

3. Verify colors match in both:
   - Footer compact cards
   - Full metric cards on page

#### Test Other Scenarios:

4. Click **"All Pass"** → All 6 metrics should have GREEN borders
5. Click **"Some Warnings"** → Should see mix of GREEN and YELLOW
6. Click **"Some Failures"** → Should see mix of GREEN, YELLOW, and RED

**Expected Results**:
- ✅ CI=0.85 displays with GREEN border
- ✅ CI=0.75 displays with YELLOW border
- ✅ CI=0.45 displays with RED border
- ✅ Threshold bar inside metric card shows value position correctly

---

### ✅ Requirement 3: "Why this score?" expands with explanation

**What to check**:
- Click on any metric card in footer → Opens modal with full details
- Full metric card has "Why this score?" button
- Clicking expands to show explainability data:
  - **Inputs Used** (name, value, source)
  - **Calculation Method** (formula)
  - **Interpretation** (plain language meaning)
  - **Recommendation** (if failing/warning)

**How to test**:

#### Method 1: Click Footer Metric
1. Click any compact metric card in the footer
2. Modal should open with full metric card
3. Click "Why this score?" button
4. Section should expand showing all explainability data

#### Method 2: View Full Cards on Test Page
1. Scroll down on test page to "Current Metrics - Full View"
2. Each metric shows as a full card
3. Click "Why this score?" to expand
4. Verify all sections display:
   - Inputs Used list with sources
   - Calculation Method in code-style box
   - Interpretation paragraph
   - Recommendation in yellow warning box (if metric not passing)

**Expected Results**:
- ✅ "Why this score?" button visible
- ✅ Clicking button expands section smoothly
- ✅ All explainability fields populated and readable
- ✅ Recommendations show for failing metrics
- ✅ Arrow icon rotates when expanded
- ✅ Modal closes with X button or clicking outside

---

### ⏳ Requirement 4: Dashboard shows radar chart

**Status**: ⚠️ **NOT YET IMPLEMENTED**

**Current State**:
- "Dashboard" button visible in footer metrics bar (right side)
- Button is placeholder - clicking does nothing yet
- Full dashboard with radar chart is **Phase 2** feature

**To Implement** (if needed now):
1. Create `MetricsDashboard` component
2. Add radar chart library (Recharts or Chart.js)
3. Implement radar chart showing all 6 metrics
4. Add history graph with threshold lines
5. Add trend analysis

**Note**: The specification says dashboard is a "nice to have" feature. Core explainability is already fully implemented in requirements 1-3.

---

## Detailed Testing Steps

### Test Flow

```
1. Open http://localhost:1420/
   ↓
2. Click "Metrics Test Page" button
   ↓
3. Verify footer shows 6 metrics
   ↓
4. Test scenario: Click "All Pass"
   → All metrics GREEN
   ↓
5. Test scenario: Click "Some Warnings"
   → Mix of GREEN and YELLOW
   ↓
6. Test scenario: Click "Some Failures"
   → Mix of GREEN, YELLOW, RED
   ↓
7. Test scenario: Click "Custom CI"
   → Slider appears
   ↓
8. Move slider to 0.85 → GREEN
9. Move slider to 0.75 → YELLOW
10. Move slider to 0.45 → RED
   ↓
11. Click CI metric in footer
   → Modal opens
   ↓
12. Click "Why this score?" button
   → Section expands with all data
   ↓
13. Verify Inputs, Calculation, Interpretation, Recommendation
   ↓
14. Close modal (X button)
   ↓
15. Scroll down to "Current Metrics - Full View"
   → See all 6 metric cards expanded
   ↓
16. Click "Why this score?" on multiple metrics
   → All expand correctly
```

---

## Visual Reference

### Footer Metrics Bar
```
┌──────────────────────────────────────────────────────────┐
│ [CI 0.78] [EV 12] [IAS 0.85] [EFI 96] [SEC 100] [PCI 0.92]│
│  YELLOW    WARN    GREEN      GREEN    GREEN     GREEN    │
│                                    [📊 Dashboard] Critical 6│
└──────────────────────────────────────────────────────────┘
```

### Expanded Metric Card
```
┌─────────────────────────────────────────────────────────┐
│ CI - Confidence Index                             0.78   │
│ Clarity and coherence of content                        │
│                                                          │
│ ● WARNING                                               │
│                                                          │
│ [════════▮═══════] ← Threshold bar                      │
│  Fail   Warn  Pass                                      │
│                                                          │
│ ▼ Why this score?                                       │
│ ┌────────────────────────────────────────────────────┐  │
│ │ Inputs Used:                                       │  │
│ │ • structural_coherence: 0.82                       │  │
│ │   (Step 3 Structural Lens)                         │  │
│ │ • term_consistency: 0.74                           │  │
│ │   (Step 5 Header Report)                           │  │
│ │                                                     │  │
│ │ Calculation Method:                                │  │
│ │ Weighted average of coherence dimensions           │  │
│ │                                                     │  │
│ │ Interpretation:                                    │  │
│ │ Content clarity is below target, primarily due to  │  │
│ │ inconsistent terminology.                          │  │
│ │                                                     │  │
│ │ ⚠ Recommendation:                                  │  │
│ │ Review Header Report and normalize terms before    │  │
│ │ proceeding.                                        │  │
│ └────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────┘
```

---

## Threshold Reference Table

| Metric | Pass | Warning | Halt | Scale |
|--------|------|---------|------|-------|
| CI | ≥ 0.80 (GREEN) | ≥ 0.70 (YELLOW) | < 0.50 (RED) | Higher = Better |
| EV | ≤ 10 (GREEN) | ≤ 20 (YELLOW) | > 30 (RED) | Lower = Better |
| IAS | ≥ 0.80 (GREEN) | ≥ 0.70 (YELLOW) | < 0.50 (RED) | Higher = Better |
| EFI | ≥ 95 (GREEN) | ≥ 90 (YELLOW) | < 80 (RED) | Higher = Better |
| SEC | = 100 (GREEN) | N/A | N/A | Must be 100 |
| PCI | ≥ 0.90 (GREEN) | ≥ 0.85 (YELLOW) | < 0.70 (RED) | Higher = Better |

---

## Common Issues

### Issue: Metrics not showing in footer
**Solution**: Verify you're on a page that includes MainLayout component (Home, RunView, MetricsTestPage).

### Issue: Colors not changing
**Solution**: Check browser console for errors. Verify Tailwind CSS is loaded.

### Issue: "Why this score?" not expanding
**Solution**: Check browser console. Verify JavaScript is enabled. Try clicking directly on the button text.

### Issue: Modal not opening
**Solution**: Click directly on the metric card. Check z-index if obscured by other elements.

---

## Verification Status

### Implemented ✅
- [x] Requirement 1: Metrics bar shows 6 metrics
- [x] Requirement 2: Color coding (green/yellow/red)
- [x] Requirement 3: "Why this score?" expandable explanation

### Pending ⏳
- [ ] Requirement 4: Dashboard with radar chart (Phase 2)

---

## Next Steps

### If Dashboard is Required Now:
1. Install charting library:
   ```bash
   npm install recharts
   # or
   npm install chart.js react-chartjs-2
   ```

2. Create `MetricsDashboard.tsx` component

3. Implement:
   - Radar chart showing all 6 metrics
   - Threshold lines on chart
   - History graph (metrics over time)
   - Trend indicators

4. Wire up Dashboard button in MetricsBar

### If Dashboard is Phase 2:
- Current implementation is **complete** for MVP
- All explainability requirements met
- Dashboard can be added later without changes to existing code

---

## Success Criteria

**MVP Requirements** (Spec: line 2971-3015):
- ✅ Every metric shows numeric value
- ✅ Threshold indicator visible
- ✅ Expandable "Why this score?" section
- ✅ Shows inputs used with sources
- ✅ Shows calculation method
- ✅ Shows plain language interpretation
- ✅ Shows actionable recommendation (when failing)

**All MVP requirements MET!** 🎉

---

## Screenshots (To Be Taken During Verification)

1. **Home Page**: Screenshot showing Metrics Test Page button
2. **Test Page - All Pass**: All metrics green
3. **Test Page - Some Warnings**: Mix of colors
4. **Test Page - Custom CI Slider**: Slider at different values
5. **Footer Metrics Bar**: Compact view
6. **Expanded Modal**: Full metric card with explainability
7. **"Why this score?" Expanded**: Showing all sections

---

## Testing Completed By

**Tester**: _______________
**Date**: _______________
**Browser**: _______________

**Checkboxes**:
- [ ] Requirement 1 verified
- [ ] Requirement 2 verified (CI at 0.85, 0.75, 0.45)
- [ ] Requirement 3 verified (expandable explanation)
- [ ] Requirement 4: Dashboard (✅ implemented / ⏳ pending)

**Notes**:
_________________________________
_________________________________
_________________________________

**Issues Found**:
_________________________________
_________________________________
_________________________________

---

**Status**: Ready for verification! Navigate to http://localhost:1420/metrics-test
