# E2BR3 SafetyDB Production Readiness Plan

Complete roadmap for bringing the E2BR3 SafetyDB system (Next.js frontend + Rust backend) to production.

## Project Status

### Current State
- **Frontend**:
  - ✅ Phase 1 Complete: Backend integration foundation ready
  - ✅ Phase 2 Complete: Full E2B(R3) form framework implemented (100%)
    - Multi-step wizard framework with all 8 sections (100%)
    - 7 reusable form components (production-ready)
    - Comprehensive validation schemas (all 8 sections)
    - CaseFormWizard fully implemented with React Hook Form (100%)
    - All form controls properly wired with useFormContext
    - Flexible step navigation (allowSkipAhead)
    - Autosave and draft functionality
  - ✅ **All Critical UX Bugs Fixed**: Date picker, required indicators, checkboxes, mobile layout
  - ✅ **Phase 3 Complete: E2B(R3) Full Compliance - 100% Type Coverage**
    - ✅ All 210 E2B(R3) data elements defined in TypeScript types
    - ✅ Complete type system matching PostgreSQL database schema (28 tables)
    - ✅ All repeating sections supported (14 repeating arrays)
    - ✅ MedDRA version tracking for all coded fields
    - ✅ All MANDATORY fields implemented
    - ✅ Section N (Message Headers), C.4.r, C.5, D.7-10, H.3.r, H.5.r integrated
    - ⚠️ Form UI exposes ~65% of fields (remaining 35% need UI components)
  - Next.js 14 application with complete type system
  - Authentication flow implemented (waiting for backend)
  - API client ready for backend integration
  - **Remaining**: Phase 4 - Complete form UI for remaining fields
- **Backend**: ✅ Complete PostgreSQL schema designed (28 tables, all 210 fields) - implementation pending
- **E2B(R3) Compliance**: ✅ **100% Type Coverage** (Form UI: 65%)
- **Production Ready**: ⚠️ PARTIAL - Requires E2B(R3) compliance for regulatory submissions
- **WCAG 2.1 AA Compliance**: 58% (needs improvement)
- **See `PHASE2-SUMMARY.md` and `E2BR3.md` for detailed status**

### Target State
- **Frontend**: Full-featured React application with backend integration
- **Backend**: Complete Rust REST API with PostgreSQL
- **E2B(R3) Compliance**: 95%+ (all mandatory fields + high-priority optional fields)
- **Production Ready**: YES (secure, validated, tested, regulatory-compliant)

---

## Phase 1: Foundation (Must Complete First)

**Goal**: Establish core infrastructure for both frontend and backend

**Duration**: 2-3 weeks

### Backend Tasks (Priority 1)

#### 1.1 Project Setup
- [ ] Initialize Rust project with Cargo
- [ ] Choose web framework (recommend: Axum for performance)
- [ ] Setup project structure (controllers, models, services, utils)
- [ ] Configure environment variables (.env setup)
- [ ] Setup logging framework (tracing crate)

#### 1.2 Database Layer
- [ ] Install PostgreSQL locally and on staging
- [ ] Choose ORM (SQLx for compile-time checks or Diesel)
- [ ] **CRITICAL**: Design E2B(R3) schema using e2br3-compliance-validator agent
  - Read documentation from `/Users/hyundonghoon/Documents`
  - Validate against ICH specifications
  - Generate compliance report
  - Implement remediation SQL
- [ ] Create initial migration files (users, organizations, base tables)
- [ ] Setup database connection pool
- [ ] Implement database models/entities
- [ ] Add audit trail table and triggers

#### 1.3 Authentication System
- [ ] Install JWT dependencies (jsonwebtoken crate)
- [ ] Install bcrypt for password hashing
- [ ] Implement user registration endpoint
- [ ] Implement login endpoint with JWT generation
- [ ] Implement refresh token mechanism
- [ ] Setup httpOnly cookie handling
- [ ] Add password reset functionality
- [ ] Implement session management

**Deliverable**: Working backend with database and auth API

### Frontend Tasks (Priority 2 - After Backend 1.1-1.3) ✅ COMPLETED

#### 1.4 API Client Setup ✅
- [x] Create `/lib/api/client.ts` with fetch wrapper
- [x] Add request interceptors (add JWT from cookies)
- [x] Add response interceptors (handle errors, refresh tokens)
- [x] Create `/lib/api/endpoints.ts` with typed endpoints
- [x] Add error handling utilities
- [x] Create uploadFile and downloadFile methods
- [x] Add comprehensive API endpoint methods for all features

#### 1.5 Type System ✅
- [x] Create `/lib/types/api.ts` with API request/response types
- [x] Create `/lib/types/e2br3.ts` matching backend schema
- [x] Create `/lib/types/common.ts` for shared types
- [x] Install Zod: `npm install zod`
- [x] Create `/lib/schemas/` directory
- [x] Create validation schemas for auth flows
- [x] Create comprehensive E2B(R3) validation schemas
- [x] Remove duplicate interfaces from component files

#### 1.6 Authentication Flow ✅
- [x] Install @tanstack/react-query and react-hot-toast
- [x] Update `app/page.tsx` (login) to call backend `/api/auth/login`
- [x] Remove localStorage auth, use httpOnly cookies
- [x] Create auth context for user state (`lib/contexts/AuthContext.tsx`)
- [x] Implement protected route hook (`lib/hooks/useProtectedRoute.ts`)
- [x] Add session check on app load
- [x] Implement logout functionality
- [x] Add auto-refresh logic for expired tokens (14 min intervals)
- [x] Create Providers component wrapping app with QueryClient and AuthProvider
- [x] Update root layout with Providers
- [x] Update dashboard layout with protected route hook
- [x] Update Sidebar to use auth context
- [x] Add toast notifications throughout

**Deliverable**: Frontend can authenticate with backend ✅

---

## Phase 2: E2B(R3) Compliance

**Goal**: Implement complete E2B(R3) data model and validation

**Duration**: 4-6 weeks

### Backend Tasks (Priority 1)

#### 2.1 Complete E2B(R3) Data Model
- [ ] Implement Section A tables (Administrative Information)
  - Message header, sender/receiver, report identification
- [ ] Implement Section B tables (Patient Information)
  - Demographics, medical history, parent info, death details
- [ ] Implement Section C tables (Reaction Information)
  - Reactions with MedDRA coding, seriousness criteria
- [ ] Implement Section D tables (Test Results)
  - Lab results, diagnostic data
- [ ] Implement Section E tables (Narrative)
  - Case narrative, reporter comments
- [ ] Implement Section F tables (Sender Information)
  - Reporter details, study information
- [ ] Implement Section G tables (Drug Information)
  - Products with WHODrug coding, dosage, route, indication
- [ ] Implement Section H tables (Dosage Regimen)
  - Administration details, dates, structured dosage

#### 2.2 Controlled Terminologies
- [ ] Create MedDRA lookup table
- [ ] Import MedDRA data (latest version)
- [ ] Create WHODrug lookup table
- [ ] Import WHODrug data (latest version)
- [ ] Create ISO country code table
- [ ] Create E2B code lists (reaction outcome, dosage form, etc.)
- [ ] Add terminology validation functions
- [ ] Create terminology search endpoints

#### 2.3 Business Rules & Validation
- [ ] Implement cardinality constraints (min/max occurrences)
- [ ] Add null flavor support (ASKU, NASK, MSK, etc.)
- [ ] Create validation service for all E2B(R3) rules
- [ ] Add cross-field validation logic
- [ ] Implement case status workflow (draft → validated → submitted)
- [ ] Add data integrity constraints

#### 2.4 Case Management API
- [ ] Create CRUD endpoints for cases
- [ ] Implement pagination and filtering
- [ ] Add search functionality
- [ ] Create validation endpoint
- [ ] Add case versioning (follow-ups, amendments)
- [ ] Implement soft deletes with audit trail

**Deliverable**: Complete backend with full E2B(R3) support

### Frontend Tasks (Priority 2 - Parallel with Backend) ✅ COMPLETED

#### 2.5 Install Form Libraries ✅
```bash
npm install react-hook-form @hookform/resolvers zod  # ✅ Installed
npm install @tanstack/react-query                    # ✅ Installed
npm install react-hot-toast                          # ✅ Installed
```

#### 2.6 Multi-Step Form Framework ✅
- [x] Design form wizard component architecture
- [x] Create `<FormWizard>` component with step navigation
- [x] Add progress indicator component (`<WizardProgress>`)
- [x] Create form context for state management (uses FormProvider from react-hook-form)
- [x] Implement step validation (per-step and whole-form validation)
- [x] Add previous/next navigation (`<WizardNavigation>`)
- [x] Create save draft functionality
- [x] Add keyboard navigation (Alt + Arrow keys)
- [x] Implement autosave with debouncing (configurable delay)

#### 2.7 E2B(R3) Form Sections ✅
- [x] Create Section A form (Administrative)
  - Safety report ID, sender/receiver, transmission date, report type
- [x] Create Section B form (Patient Info)
  - Demographics (initials, DOB, sex), age, weight, height
- [x] Create Section C form (Reactions)
  - Reaction description with MedDRA autocomplete, dates, outcome, seriousness criteria
- [x] Create Section D form (Test Results) - Optional
  - Test date, name, result, unit, range, comments
- [x] Create Section E form (Narratives)
  - Case narrative (10,000 char limit), sender diagnosis, reporter comments
- [x] Create Section F form (Sender/Reporter)
  - Qualification, name, organization, contact info, full address
- [x] Create Section G form (Drugs)
  - Drug characterization, product info with WHODrug autocomplete, indication, action taken
- [x] Create Section H form (Dosage) - Optional
  - Administration dates, dosage description, structured dosage info (value, unit, route, frequency)
- [x] Integrated all sections into `<CaseFormWizard>` component
- [x] Connected CaseFormWizard to `/app/dashboard/cases/page.tsx`

#### 2.8 Reusable Form Components ✅
- [x] Create `<FormInput>` with validation and tooltips
- [x] Create `<FormSelect>` with search
- [x] Create `<FormDatePicker>` with calendar picker
- [x] Create `<FormAutocomplete>` for terminologies (MedDRA, WHODrug)
- [x] Create `<FormTextarea>` with character counter
- [x] Create `<FormCheckbox>` with label
- [x] Create `<FormRadioGroup>` for radio button groups
- [x] Add error message components (integrated in all form fields)
- [x] Export all components from `/components/forms/index.ts`

#### 2.9 Validation Enhancement ✅
- [x] Create Zod schemas for all E2B(R3) sections in `/lib/schemas/e2br3.ts`
- [x] Implement field-level validation (onChange mode)
- [x] Add real-time validation feedback (instant error display)
- [x] Create cross-field validation rules (supported in wizard)
- [x] Add server-side validation error display (FormMessage component)
- [x] Implement autosave with debouncing (default 2000ms, configurable)

#### 2.10 Terminology Integration ✅
- [x] Create MedDRA autocomplete using `<FormAutocomplete>`
- [x] Create WHODrug autocomplete using `<FormAutocomplete>`
- [x] Add country code selector (static options in FormSelect)
- [x] Add code list dropdowns (outcome, severity, qualification, etc.)
- [x] Implement search with debouncing (300ms default)
- [x] Add loading states for async terminology lookups
- [x] Connected to backend API endpoints:
  - `api.terminology.searchMedDRA(query)`
  - `api.terminology.searchWhoDrug(query)`

**Deliverable**: Complete forms for all E2B(R3) sections ✅ DELIVERED

**E2B(R3) Form Coverage**: ~90% (all 8 sections implemented with comprehensive fields)

#### 2.11 UX Testing & Bug Fixes ✅ COMPLETED

**Comprehensive UX Testing Completed** (using ux-playwright-tester agent):
- ✅ Tested all 8 E2B(R3) form sections
- ✅ Verified step navigation and allowSkipAhead functionality
- ✅ Tested form validation for required fields
- ✅ Checked autocomplete fields (MedDRA, WHODrug)
- ✅ Verified keyboard navigation and accessibility
- ✅ Tested edge cases and error scenarios
- ✅ WCAG 2.1 AA compliance assessment: 58%

**All Critical Bugs Fixed** ✅:
1. **Date Picker Type Mismatch** ✅ FIXED
   - Solution: Updated FormDatePicker to convert Date to ISO string (YYYY-MM-DD format)
   - File: `components/forms/FormDatePicker.tsx:164-166`
   - Impact: Date validation now works correctly

2. **Missing Required Field Indicators** ✅ FIXED
   - Solution: Added `required` prop to age value and unit fields
   - File: `components/case-form/CaseFormWizard.tsx:143,150`
   - Impact: Users now see which fields are mandatory

3. **Checkbox Validation Error** ✅ FIXED
   - Solution: Set proper defaultValues for all boolean fields in seriousness criteria
   - File: `components/case-form/CaseFormWizard.tsx:826-839`
   - Impact: Checkboxes now default to false, validation works correctly

4. **Mobile Layout Broken** ✅ FIXED
   - Solution: Implemented responsive sidebar with hamburger menu and backdrop
   - Files: `components/Sidebar.tsx`, `app/dashboard/layout.tsx`
   - Impact: Application now fully usable on mobile devices
   - Features: Slide-in menu, backdrop overlay, mobile header

**Medium Priority Issues (Future Enhancement)**:
- Improve error message clarity (6 instances) - Can be addressed in Phase 3
- Add loading states for async operations - Partially implemented
- Improve focus management between steps - Working, can be enhanced
- Add more comprehensive ARIA labels - Target: 80% WCAG 2.1 AA compliance

**Production Readiness**: ✅ READY - All critical bugs resolved, frontend ready for backend integration

---

## Phase 3: E2B(R3) Full Compliance (CRITICAL)

**Goal**: Achieve 95%+ E2B(R3) field coverage and correct all structural issues

**Duration**: 10-15 days

**Priority**: BLOCKING - Must complete before regulatory submissions

**Reference**: See `E2BR3.md` for complete E2B(R3) specification and gap analysis

### Overview

The current implementation has **50% E2B(R3) compliance** with critical structural issues:
- Section labeling does NOT match ICH specification (A-H labels are incorrect)
- 105 of 210 data elements missing
- 23 MANDATORY fields missing
- Misplaced sections (C.2.r in Section F, G.k.4.r in Section H)
- Missing Section N (Batch/Message Headers) entirely

### Frontend Tasks (Priority 1 - CRITICAL)

#### 3.1 Structural Corrections (Day 1-3)

**Goal**: Fix section labeling and hierarchy to match E2B(R3) specification

**Tasks**:
- [ ] **Rename all section labels** to match E2B(R3):
  - Current "Section A" → **Section C** (Case Identification)
  - Current "Section B" → **Section D** (Patient Information)
  - Current "Section C" → **Section E** (Reaction Information)
  - Current "Section D" → **Section F** (Test Results)
  - Current "Section E" → **Section H** (Case Narrative)
  - Current "Section F" → Move to **Section C.2.r** (Primary Source - part of Case ID)
  - Current "Section G" → Keep as **Section G** (Drug Information) ✅ Correct
  - Current "Section H" → Move to **Section G.k.4.r** (Dosage - part of Drug Info)

- [ ] **Reorganize form structure**:
  - Move Primary Source (current F) into Case Identification section (C)
  - Merge Dosage Regimen (current H) into Drug Information section (G)
  - Update step order in CaseFormWizard
  - Update wizard progress labels

- [ ] **Add Section N** (Batch/Message Headers):
  - Create optional pre-step for batch transmission
  - Fields: Batch Number, Sender/Receiver IDs, Transmission Date

- [ ] **Update files**:
  - `/components/case-form/CaseFormWizard.tsx`
  - `/lib/types/e2br3.ts`
  - `/lib/schemas/e2br3.ts`

**Deliverable**: Correct section structure matching E2B(R3) specification

#### 3.2: Mandatory Field Implementation (Day 4-7)

**Goal**: Implement all 23 missing MANDATORY fields

**Tasks**:

**Section C - Case Identification** (6 missing):
- [ ] C.1.2: Date of Creation (Date/Time, 1..1)
- [ ] C.1.4: Date Report Was First Received from Source (Date/Time, 1..1)
- [ ] C.1.5: Date of Most Recent Information (Date/Time, 1..1)
- [ ] C.1.7: Does This Case Fulfil Expedited Report Criteria? (Boolean, 1..1)
- [ ] C.3.1: Sender Type (1N coded, 1..1)
  - Code list: 1=Pharma, 2=Regulatory Authority, 3=Health Professional, 4=Regional Centre, 5=WHO, 6=Other
- [ ] C.3.2: Sender's Organisation (full details, not just name)

**Section D - Patient Information** (1 missing):
- [ ] D.1: Patient Name or Initials (currently partial - add full name support with null flavor)

**Section C.2.r - Primary Source** (1 missing):
- [ ] C.2.r.5: Primary Source for Regulatory Purposes (1N, 0..1)

**Tasks**:
- [ ] Add fields to type definitions (`/lib/types/e2br3.ts`)
- [ ] Add validation schemas (`/lib/schemas/e2br3.ts`)
- [ ] Add form controls to appropriate steps
- [ ] Mark fields as required in UI
- [ ] Add tooltips explaining each field

**Deliverable**: All mandatory fields implemented with validation

#### 3.3: Repeating Sections Support (Day 8-10)

**Goal**: Implement proper array handling for repeating sections

**Current Issue**: Many ".r" (repeatable) sections implemented as single-item only

**Tasks**:

- [ ] **C.4.r**: Literature References (0..*)
  - Create array UI component
  - Add/remove/reorder buttons
  - Validate each entry

- [ ] **C.5.1.r**: Study Registration Numbers (0..*)
  - Currently single field
  - Support multiple registrations per country

- [ ] **D.9.2.r**: Reported Cause(s) of Death (0..*)
  - Currently single cause
  - Support multiple causes with MedDRA codes
  - Add MedDRA version field

- [ ] **D.9.4.r**: Autopsy Cause(s) of Death (0..*)
  - Currently single cause
  - Support multiple causes with MedDRA codes

- [ ] **G.k.6.r**: Drug Indication(s) (0..*)
  - Currently single indication
  - Support multiple indications per drug

- [ ] **H.3.r**: Sender's Diagnoses (0..*)
  - Currently single diagnosis
  - Support multiple diagnoses with MedDRA codes

**Create Reusable Array Field Component**:
- [ ] `<FormArrayField>` component
  - Add button (+ icon)
  - Remove button (trash icon)
  - Drag handles for reorder
  - Validation per item
  - Visual separators between items

**Deliverable**: All repeating sections support multiple entries

#### 3.4: Code List Compliance (Day 11-12)

**Goal**: Replace free text with E2B(R3) code lists where required

**Tasks**:

- [ ] **D.2.3**: Patient Age Group → Use code list (1-6)
  - 1=Neonate, 2=Infant, 3=Child, 4=Adolescent, 5=Adult, 6=Elderly

- [ ] **D.2.2b**: Age Unit → Ensure code list compliance
  - 800=Decade, 801=Year, 802=Month, 803=Week, 804=Day, 805=Hour

- [ ] **G.k.8.r.1**: Drug Recurrence Readministration → Use code list
  - Replace text field with dropdown

- [ ] **G.k.8.r.2**: Drug Recurrence → Use code list
  - Replace text field with dropdown

- [ ] **Add MedDRA Version Tracking** for all MedDRA fields:
  - E.i.2.1a: MedDRA Version for Reaction
  - F.r.2.1a: MedDRA Version for Test Name
  - G.k.6.r.2a: MedDRA Version for Indication
  - H.3.r.1a: MedDRA Version for Diagnosis
  - D.7.1.r.1a: MedDRA Version for Medical History
  - D.9.2.r.1a: MedDRA Version for Cause of Death
  - D.9.4.r.1a: MedDRA Version for Autopsy Cause

**Create Version Selector Component**:
- [ ] `<MedDRAVersionSelect>` component
  - Dropdown with MedDRA versions (e.g., "25.1", "25.0", "24.1")
  - Auto-detect latest version
  - Store version with code

**Add HL7 OID Mappings**:
- [ ] Create OID constants file (`/lib/constants/oids.ts`)
- [ ] Map each code system to its OID
  - MedDRA: `2.16.840.1.113883.6.163`
  - WHODrug: (varies by implementation)
  - ISO 3166 Country: `1.0.3166.1.2.2`

**Deliverable**: All coded fields use proper code lists with versions

#### 3.5: High-Priority Optional Fields (Day 13-15)

**Goal**: Implement frequently-required optional fields

**Tasks**:

**Case Identification**:
- [ ] C.1.8.1: Worldwide Unique Case Identification Number (100AN, 0..1)
- [ ] C.1.11.2: Reason for Nullification/Amendment (2000AN, 0..1)
- [ ] C.3.3: Person Responsible for Sending (full details)

**Patient Information**:
- [ ] D.2.2.1: Gestation Period (for fetal cases, 0..1)
  - D.2.2.1a: Value (Real)
  - D.2.2.1b: Unit (1N: trimester, week, day)
- [ ] D.7.2: Text for Relevant Medical History (10000AN, 0..1)
- [ ] D.7.3: Concomitant Therapies (10000AN, 0..1)

**Reaction Information**:
- [ ] E.i.1.2: Reaction as Reported (translation, 250AN, 0..*)
- [ ] E.i.9: Country Where Reaction Occurred (2A, 0..*)

**Drug Information**:
- [ ] G.k.2.3.r: Substance / Specified Substance (0..*)
  - G.k.2.3.r.1: Name (250AN)
  - G.k.2.3.r.2: TermID (8N)
  - G.k.2.3.r.3a: Strength (number)
  - G.k.2.3.r.3b: Strength (unit)
- [ ] G.k.2.4.r.1: MPID Version Date/Number (Date/8N, 0..*)
- [ ] G.k.4.r.5-6: Cumulative Dose to First Reaction
  - G.k.4.r.5: Value (Real)
  - G.k.4.r.6: Unit (50AN)

**Narrative**:
- [ ] H.4: Sender's Comments (2000AN, 0..1)

**Deliverable**: High-priority optional fields implemented

#### 3.6: Validation & Testing (Throughout)

**Tasks**:
- [ ] Update Zod schemas for all new fields
- [ ] Add client-side validation messages
- [ ] Test all new fields with various inputs
- [ ] Verify null flavor support works
- [ ] Test repeating sections (add/remove/reorder)
- [ ] Verify code lists populate correctly
- [ ] Test MedDRA version tracking
- [ ] Run full form submission test
- [ ] Validate against backend API (if available)

**Testing Checklist**:
- [ ] All mandatory fields show required indicators
- [ ] Validation errors display correctly
- [ ] Repeating sections add/remove without errors
- [ ] Code lists match E2B(R3) specification
- [ ] MedDRA versions save correctly
- [ ] Form submission includes all new fields
- [ ] Autosave works with new fields
- [ ] Draft save/load works correctly

**Deliverable**: 95%+ E2B(R3) compliance achieved

### Backend Tasks (Priority 2 - Parallel)

**Note**: Backend must update database schema and API to support new fields

#### 3.7: Database Schema Updates
- [ ] Add missing mandatory fields to database
- [ ] Create tables for repeating sections
- [ ] Add MedDRA version tracking columns
- [ ] Add code list validation constraints
- [ ] Create migration scripts
- [ ] Test migrations on dev database

#### 3.8: API Updates
- [ ] Update case model to include new fields
- [ ] Update validation logic for mandatory fields
- [ ] Add support for repeating sections in API
- [ ] Update JSON serialization/deserialization
- [ ] Test API with new field structure
- [ ] Update API documentation

**Deliverable**: Backend supports all new E2B(R3) fields

### Compliance Verification

After Phase 3 completion, verify compliance:

- [ ] Run through complete form with all 8 sections
- [ ] Verify all 23 mandatory fields are implemented
- [ ] Test repeating sections with multiple entries
- [ ] Validate code lists against E2B(R3) specification
- [ ] Check MedDRA version tracking works
- [ ] Verify section labeling matches ICH specification
- [ ] Generate compliance report (target: 95%+)
- [ ] Document remaining gaps (5% acceptable)

**Success Criteria**:
- ✅ All section labels match E2B(R3) specification
- ✅ All 23 mandatory fields implemented
- ✅ All repeating sections support multiple entries
- ✅ All coded fields use proper code lists
- ✅ MedDRA version tracking works
- ✅ 95%+ E2B(R3) compliance achieved
- ✅ Form suitable for regulatory submissions

**Deliverable**: Production-ready E2B(R3) compliant form

---

## Phase 4: XML Processing & Advanced Features

**Goal**: Full E2B(R3) XML import/export with validation

**Duration**: 3-4 weeks

**Prerequisites**: Phase 3 (E2B(R3) Full Compliance) must be complete

### Backend Tasks (Priority 1)

#### 4.1 XML Processing Setup
- [ ] Install XML libraries (quick-xml or xml-rs)
- [ ] Download ICH XSD schemas from `/Users/hyundonghoon/Documents/4_ICH_ICSR_Schema_Files/`
- [ ] Setup XSD validation (using libxml2 bindings or similar)
- [ ] Create XML utilities module

#### 4.2 XML Generation
- [ ] Implement E2B(R3) XML generator
- [ ] Create ICH message header builder
- [ ] Map database models to XML structure (all sections C-H)
- [ ] Implement null flavor handling in XML
- [ ] Add controlled terminology code insertion
- [ ] Validate generated XML against XSD
- [ ] Create export endpoint: `GET /api/xml/export/:case_id`

#### 4.3 XML Import/Parsing
- [ ] Implement E2B(R3) XML parser
- [ ] Validate incoming XML against XSD
- [ ] Map XML to database models
- [ ] Handle missing/null data (null flavors)
- [ ] Validate controlled terminologies
- [ ] Create import endpoint: `POST /api/xml/import`
- [ ] Generate validation report

#### 4.4 XML Testing
- [ ] Test with ICH example 1: Literature initial report
- [ ] Test with ICH example 2: Nullification
- [ ] Test with ICH example 3: New number initial
- [ ] Test with ICH example 4: Follow-up report
- [ ] Test with ICH example 5: Clinical trial
- [ ] Verify round-trip (import → export → compare)
- [ ] Test XSD validation catches all errors

**Deliverable**: Full XML import/export with validation

### Frontend Tasks (Priority 2)

#### 4.5 XML Export Interface
- [ ] Update `app/dashboard/submission/page.tsx`
- [ ] Add export button with loading state
- [ ] Call backend export endpoint
- [ ] Download XML file to user's machine
- [ ] Show validation status before export
- [ ] Add preview of XML (syntax highlighted)

#### 4.6 XML Import Interface
- [ ] Update `app/dashboard/import/page.tsx`
- [ ] Add file upload dropzone
- [ ] Validate file type (.xml only)
- [ ] Implement file size limits (e.g., 10MB max)
- [ ] Call backend import endpoint with multipart/form-data
- [ ] Display validation results
- [ ] Show imported case data for review
- [ ] Add "Save Imported Case" button

#### 4.7 Dashboard Enhancements
- [ ] Update `app/dashboard/page.tsx` with real data from backend
- [ ] Fetch case statistics (total, by status, by severity)
- [ ] Implement charts with Recharts (case trends over time)
- [ ] Add recent cases list
- [ ] Create alerts for validation errors
- [ ] Add quick actions (create case, import XML)

**Deliverable**: Full XML workflow UI

---

## Phase 5: Security & Production Hardening

**Goal**: Production-ready security, testing, and deployment

**Duration**: 3-4 weeks

**Prerequisites**: Phase 4 (XML Processing) must be complete

### Backend Tasks (Priority 1)

#### 5.1 Security Enhancements
- [ ] Implement RBAC (roles: Admin, Manager, User, Viewer)
- [ ] Add organization-level data isolation
- [ ] Implement CSRF protection
- [ ] Add rate limiting (using tower-middleware or similar)
- [ ] Setup request size limits
- [ ] Implement input sanitization
- [ ] Add SQL injection prevention checks (verify parameterized queries)
- [ ] Configure CORS for frontend domain only
- [ ] Setup secure headers (HSTS, X-Frame-Options, etc.)

#### 5.2 Testing
- [ ] Write unit tests for all business logic functions
- [ ] Write integration tests for all API endpoints
- [ ] Create test fixtures for E2B(R3) data
- [ ] Test against all 5 ICH examples
- [ ] Add load testing (using k6 or similar)
- [ ] Test concurrent user scenarios
- [ ] Verify audit trail captures all changes
- [ ] Test rollback procedures

#### 4.3 Monitoring & Logging
- [ ] Implement structured logging with tracing crate
- [ ] Add log levels (error, warn, info, debug)
- [ ] Setup error tracking (Sentry or similar)
- [ ] Add performance monitoring (response times)
- [ ] Create health check endpoint (`GET /health`)
- [ ] Add database connection monitoring
- [ ] Setup alerts for critical errors

#### 4.4 Database Optimization
- [ ] Add indexes on frequently queried fields
- [ ] Optimize slow queries (use EXPLAIN ANALYZE)
- [ ] Setup connection pooling configuration
- [ ] Implement query result caching (Redis optional)
- [ ] Add database backup strategy
- [ ] Create migration rollback scripts

#### 4.5 Deployment Preparation
- [ ] Create Dockerfile for backend
- [ ] Setup multi-stage builds
- [ ] Create docker-compose for local dev
- [ ] Write deployment documentation
- [ ] Setup CI/CD pipeline (GitHub Actions)
- [ ] Configure staging environment
- [ ] Create database migration strategy for prod
- [ ] Setup secrets management (environment variables)

**Deliverable**: Production-ready backend

### Frontend Tasks (Priority 2)

#### 4.6 Authentication & Security
- [ ] Remove ALL localStorage usage for sensitive data
- [ ] Verify JWT stored only in httpOnly cookies
- [ ] Implement CSRF token handling (if backend uses)
- [ ] Add session timeout warning (before token expires)
- [ ] Implement auto-logout on inactivity
- [ ] Add secure route guards for all protected pages
- [ ] Verify no sensitive data in URL params/query strings

#### 4.7 Error Handling & UX
- [ ] Create global error boundary component
- [ ] Add error pages (404, 500, 403)
- [ ] Implement toast notifications for all actions
- [ ] Add loading skeletons for all async operations
- [ ] Create retry mechanisms for failed requests
- [ ] Add offline detection and messaging
- [ ] Implement optimistic UI updates where appropriate

#### 4.8 Testing
- [ ] Install testing libraries:
  ```bash
  npm install -D jest @testing-library/react @testing-library/jest-dom
  npm install -D @testing-library/user-event
  ```
- [ ] Write unit tests for utility functions
- [ ] Write component tests for forms
- [ ] Write integration tests for user flows
- [ ] Test all E2B(R3) form validations
- [ ] Test XML import/export workflows
- [ ] Add E2E tests (Playwright or Cypress)

#### 4.9 Accessibility
- [ ] Add ARIA labels to all interactive elements
- [ ] Ensure keyboard navigation works throughout
- [ ] Test with screen reader (NVDA or JAWS)
- [ ] Add focus management for modals and wizards
- [ ] Verify color contrast meets WCAG 2.1 AA
- [ ] Add skip navigation links
- [ ] Test with keyboard-only navigation

#### 4.10 Performance Optimization
- [ ] Implement code splitting for routes
- [ ] Add lazy loading for heavy components
- [ ] Optimize images (use next/image)
- [ ] Minimize bundle size (analyze with webpack-bundle-analyzer)
- [ ] Add React Query caching strategies
- [ ] Implement virtual scrolling for long lists
- [ ] Add Web Vitals monitoring
- [ ] Optimize Tailwind CSS (purge unused styles)

#### 4.11 Deployment Preparation
- [ ] Create production build: `npm run build`
- [ ] Test production build locally: `npm start`
- [ ] Create Dockerfile for frontend
- [ ] Configure environment variables for production
- [ ] Setup CI/CD pipeline (GitHub Actions)
- [ ] Configure staging environment
- [ ] Add deployment documentation

**Deliverable**: Production-ready frontend

---

## Phase 5: Regulatory Compliance & Final Validation

**Goal**: Ensure full E2B(R3) compliance and regulatory readiness

**Duration**: 2-3 weeks

### Backend Tasks

#### 5.1 E2B(R3) Final Validation
- [ ] **Run e2br3-compliance-validator agent on final schema**
- [ ] Verify 100% coverage of mandatory fields
- [ ] Verify all cardinality constraints implemented
- [ ] Verify all controlled terminologies validated
- [ ] Verify null flavor handling correct
- [ ] Generate final compliance report
- [ ] Document any known limitations

#### 5.2 XML Validation
- [ ] Validate all generated XML against XSD (100% pass rate)
- [ ] Test with regional requirements (FDA, EMA, PMDA)
- [ ] Verify message headers match specifications
- [ ] Test nullification and follow-up workflows
- [ ] Document XML generation process

#### 5.3 Audit & Compliance
- [ ] Verify audit trail captures ALL data changes
- [ ] Test audit log immutability
- [ ] Implement data retention policies
- [ ] Add audit log export functionality
- [ ] Document compliance with 21 CFR Part 11 (if FDA submission)
- [ ] Create data privacy documentation (GDPR if applicable)

### Frontend Tasks

#### 5.4 User Acceptance Testing
- [ ] Create UAT test plan
- [ ] Test all user workflows end-to-end
- [ ] Verify all validation messages clear and helpful
- [ ] Test with real-world case data
- [ ] Gather feedback from actual pharmacovigilance users
- [ ] Fix usability issues identified

#### 5.5 Documentation
- [ ] Create user manual
- [ ] Document all form fields with E2B(R3) references
- [ ] Create video tutorials for common workflows
- [ ] Document error messages and resolutions
- [ ] Create admin guide

### Combined Tasks

#### 5.6 Integration Testing
- [ ] Test complete case lifecycle (create → validate → export → submit)
- [ ] Test XML round-trip with all 5 ICH examples
- [ ] Test multi-user concurrent editing
- [ ] Test case versioning and follow-ups
- [ ] Verify organization data isolation
- [ ] Load test entire system (frontend + backend)

#### 5.7 Security Audit
- [ ] Run OWASP ZAP or similar security scanner
- [ ] Test for common vulnerabilities (SQL injection, XSS, CSRF)
- [ ] Verify all sensitive data encrypted in transit
- [ ] Test authentication bypass attempts
- [ ] Verify session management secure
- [ ] Document security measures

**Deliverable**: Fully compliant, tested system ready for production

---

## Dependencies & Critical Path

### Critical Path (must be done in order)

1. **Backend Database Schema** (Phase 1.2)
   - BLOCKS: All backend data operations
   - BLOCKS: Frontend type definitions
   - **Use e2br3-compliance-validator before proceeding**

2. **Backend Authentication** (Phase 1.3)
   - BLOCKS: Frontend authentication integration
   - BLOCKS: All protected API endpoints

3. **Backend Case API** (Phase 2.4)
   - BLOCKS: Frontend forms can save data
   - BLOCKS: Frontend dashboard shows real data

4. **Backend XML Processing** (Phase 3.2, 3.3)
   - BLOCKS: Frontend XML import/export
   - BLOCKS: E2B(R3) compliance validation

### Parallel Work Opportunities

- **Phase 2**: Frontend forms (2.6-2.10) can be built in parallel with backend data model (2.1-2.4)
- **Phase 3**: Frontend XML UI (3.5-3.7) can be built while backend XML processing (3.1-3.4) is in progress
- **Phase 4**: Frontend testing/optimization (4.7-4.10) can happen in parallel with backend hardening (4.1-4.4)

---

## Milestones & Timeline

| Milestone | Duration | Deliverable |
|-----------|----------|-------------|
| **M1: Foundation** | 3 weeks | Working backend with auth, frontend can login |
| **M2: E2B(R3) Data** | 6 weeks | Complete data model, all forms functional |
| **M3: XML Processing** | 4 weeks | Full XML import/export working |
| **M4: Production Ready** | 4 weeks | Secure, tested, deployed to staging |
| **M5: Validated** | 3 weeks | Compliance verified, UAT complete |
| **TOTAL** | **~20 weeks** | Production deployment |

---

## Success Criteria

### E2B(R3) Compliance
- ✅ 100% of mandatory fields implemented
- ✅ All controlled terminologies validated (MedDRA, WHODrug)
- ✅ All cardinality constraints enforced
- ✅ XSD validation passes for all generated XML
- ✅ All 5 ICH test cases can be imported and exported

### Security
- ✅ JWT authentication with httpOnly cookies
- ✅ RBAC with organization isolation
- ✅ No critical vulnerabilities (OWASP Top 10)
- ✅ Audit trail for all data changes
- ✅ TLS 1.3 for all connections

### Performance
- ✅ API response time < 200ms for 95th percentile
- ✅ Frontend page load < 2s on 3G connection
- ✅ Support 100 concurrent users
- ✅ Database queries optimized (< 50ms avg)

### Testing
- ✅ 80%+ code coverage (backend)
- ✅ All critical user flows tested (E2E)
- ✅ Load testing passes (1000 requests/min)
- ✅ All accessibility checks pass (WCAG 2.1 AA)

---

## Risk Mitigation

| Risk | Impact | Mitigation |
|------|--------|------------|
| **E2B(R3) schema complexity** | High | Use e2br3-compliance-validator agent early and often |
| **XML validation failures** | High | Test against all 5 ICH examples throughout development |
| **Performance at scale** | Medium | Load test early, optimize queries proactively |
| **Terminology data unavailable** | Medium | Use mock data initially, plan for licensed MedDRA/WHODrug |
| **Frontend/backend type mismatches** | Medium | Generate TypeScript types from backend schema |
| **Scope creep** | High | Strictly follow E2B(R3) spec, defer nice-to-haves |

---

## Next Steps

1. **Start with Backend Phase 1.2**: Design database schema
2. **Use e2br3-compliance-validator agent**: BEFORE writing any migration code
3. **Set up CI/CD early**: Automate testing from the start
4. **Weekly reviews**: Check progress against this plan
5. **Update this plan**: As you learn more about requirements

**Ready to begin? Start with BACKEND.md Phase 1 tasks.**
