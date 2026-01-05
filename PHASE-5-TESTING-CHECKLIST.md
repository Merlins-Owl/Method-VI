# Phase 5 Frontend Integration - Manual Testing Checklist

**Status**: Dev server running at http://localhost:1420
**Date**: 2026-01-02

## Prerequisites
- ✅ `npm run tauri dev` running successfully
- ✅ Application window open
- ✅ No TypeScript errors in Phase 5 code

---

## Basic Integration Tests

### StatusBar Visibility
- [ ] **App launches successfully**
  - Application window opens without crashes
  - No console errors on startup

- [ ] **StatusBar visible in RunView**
  - Navigate to a run (create new run if needed)
  - StatusBar appears in header (between run info and navigation)
  - Contains ModeBadge and CalloutBadge separated by divider

---

## ModeBadge UX Guardrails

### Initial State (Before Step 2)
- [ ] **Shows "Detecting..." initially** (NOT "Pending")
  - Badge displays: "Mode: Detecting..."
  - Gray styling with border

- [ ] **Tooltip shows helpful message**
  - Hover over "Detecting..." badge
  - Tooltip: "Mode is determined after Step 2 baseline analysis. Keep working!"

### After Step 2 Completion
- [ ] **Mode detected and displayed**
  - Badge shows mode name: "Architecting", "Builder", or "Refining"
  - Color-coded: purple (Architecting), blue (Builder), green (Refining)
  - Confidence percentage shown (e.g., "Architecting (85%)")

- [ ] **Lock icon has tooltip**
  - If mode is locked, 🔒 icon appears
  - Hover over lock icon
  - Tooltip: "Mode is fixed for this run. You can continue editing."

### Details Popover
- [ ] **Expands to show details**
  - Click ModeBadge
  - Popover appears below badge (z-40)
  - Shows: Mode, CI Baseline, Confidence, Status, User Message
  - All null values handled gracefully (no .toFixed() errors)

---

## CalloutPanel UX Guardrails

### Badge Display
- [ ] **Badge updates automatically**
  - Callout count updates every 5 seconds
  - Color matches highest severity tier
  - Shows "X pending" count in red badge if Critical unacknowledged

- [ ] **Opens on click**
  - Click CalloutBadge
  - CalloutPanel modal opens (z-50)
  - Backdrop visible and clickable to close
  - ESC key closes panel

### Panel Content
- [ ] **Groups by human-friendly labels** (NOT raw tier names)
  - Critical callouts shown as "Must Review"
  - Warning callouts shown as "Important"
  - Attention callouts shown as "Minor"
  - Info callouts shown as "Info"

- [ ] **Callout cards display correctly**
  - Metric name shown
  - Explanation text (uses `explanation` field)
  - Recommendation with 💡 icon
  - Current value displayed
  - **Uses "Change:" label** (NOT "Delta:")

### Acknowledgment Flow
- [ ] **Acknowledge requires checkbox** (intentional friction)
  - Critical callout shows "I understand" checkbox
  - Acknowledge button DISABLED until checkbox checked
  - Clicking Acknowledge without checkbox does nothing

- [ ] **Individual acknowledgment works**
  - Check "I understand" on a callout
  - Click "Acknowledge" button
  - Callout marked with "✓ Acknowledged"
  - No longer requires acknowledgment

- [ ] **Bulk acknowledgment works**
  - Multiple Critical callouts pending
  - Footer shows "X callouts pending acknowledgment"
  - "I have reviewed these concerns..." checkbox appears
  - "Acknowledge All" button DISABLED until checkbox checked
  - Check confirmation checkbox
  - Click "Acknowledge All"
  - All callouts acknowledged

---

## Gate Integration Tests

### Before Acknowledgment
- [ ] **Gate BLOCKED when Critical unacknowledged**
  - Navigate to a gate (Step 0 → Step 1 transition)
  - Click gate approval
  - Alert shows: "Please acknowledge Critical callouts before proceeding"
  - Gate approval cancelled
  - User returns to current step

### After Acknowledgment
- [ ] **Gate ALLOWED after acknowledgment**
  - Open CalloutPanel
  - Acknowledge all Critical callouts
  - Close panel
  - Click gate approval again
  - Confirmation dialog appears
  - Gate proceeds to next step

### Error Handling
- [ ] **Fail-open on check error**
  - If `can_proceed` check fails (backend error)
  - Error logged to console
  - Gate approval continues anyway (doesn't block user)

---

## Auto-Refresh Behavior

### Polling Intervals
- [ ] **CalloutBadge updates every 5 seconds**
  - Create callout in backend
  - Badge count updates within 5 seconds
  - No manual refresh needed

- [ ] **ModeBadge updates every 5 seconds**
  - Mode detection completes
  - Badge updates within 5 seconds
  - Shows new mode info

---

## Visual/UI Tests

### Styling Consistency
- [ ] **Matches existing dark theme**
  - Gray backgrounds (gray-800, gray-900)
  - Gray borders (gray-700, gray-600)
  - Proper contrast for text

- [ ] **Tier colors distinct**
  - Critical: Red (bg-red-900/30, border-red-500)
  - Warning: Orange (bg-orange-900/30, border-orange-500)
  - Attention: Yellow (bg-yellow-900/30, border-yellow-500)
  - Info: Blue (bg-blue-900/30, border-blue-500)

### Z-Index Layering
- [ ] **Correct stacking order**
  - ModeBadge popover (z-40) below modals
  - CalloutPanel modal (z-50) above everything
  - Backdrop click closes panel
  - No z-index conflicts

---

## Edge Cases

### Null Safety
- [ ] **Handles null ModeInfo fields**
  - No errors when mode is null
  - No errors when ci_baseline is null
  - No errors when confidence is null
  - No .toFixed() crashes on null values

### Empty States
- [ ] **Zero callouts displays correctly**
  - Badge shows "No callouts" with green dot
  - Panel shows "No callouts to display"

### Loading States
- [ ] **Loading indicators work**
  - Initial load shows "Loading..." badge
  - Acknowledging shows "Acknowledging..." or "..."

---

## Console Checks

### Expected Logs
- [ ] **No React errors in console**
- [ ] **Tauri commands log correctly**
  - `get_callout_summary` calls logged
  - `get_current_mode` calls logged
  - `can_proceed` check logged at gate

### Expected Warnings (OK to ignore)
- [ ] **Rust warnings** (56 warnings - all pre-existing)
  - Naming conventions (Step0_Intent, etc.)
  - Unused database functions
  - Deprecated halt/pause check methods

---

## Final Verification

- [ ] **All basic tests pass**
- [ ] **All ModeBadge UX guardrails verified**
- [ ] **All CalloutPanel UX guardrails verified**
- [ ] **Gate blocking works correctly**
- [ ] **No runtime errors**
- [ ] **No UI glitches or layout issues**

---

## Issues Found

*Document any issues discovered during testing here:*

1.

2.

3.

---

## Sign-Off

**Tester**: _________________
**Date**: _________________
**Result**: ☐ PASS  ☐ FAIL (with issues documented above)
