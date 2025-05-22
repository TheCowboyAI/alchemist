# KÉCŌ Capital Loan Processing Flow

This diagram represents the primary loan application and approval process flow extracted from the KECO_v6_04012025.json data model.

## Process Overview

The diagram shows the progression from initial loan application through document processing, underwriting, and final loan closing. It highlights the key steps and decision points in the KÉCŌ Capital private lending workflow.

```mermaid
flowchart TD
    %% Main Actors
    A[Borrower] 
    B[Online Portal]
    C[Processor/AI]
    D[Underwriting]
    E[Title/Attorney]

    %% Primary Flow Nodes
    n3[Borrower File Created]
    n2[Deal File Created]
    n117[Initial Term Outline Generated]
    n118[ITO Selected]
    n119[Documents Requested]
    n1[Timer Started]
    n7[File Submitted to TPO]
    n18[Underwriting Decision Made]
    n64[Conditions Met]
    n68[CLA Issued]
    n70[CTC Signed by Borrower]
    n25[Docs Signed]
    n26[Loan Recorded]
    n27[KECO Payment Received]
    
    %% Alternative Flow
    n6[File Terminated]
    
    %% Document Verification Nodes
    subgraph Document_Processing [Document Processing]
        n56[Borrower Submits Full Loan Application]
        n21[Soft Credit/Background Check Completed]
        n58[Entity Documents Collected & Reviewed]
        n60[Bank Statements & Liquidity Verified]
        n62[Final Loan Structuring Completed]
    end
    
    %% Main Flow
    A -->|Submits Info| n3
    A -->|Submits Address & Loan Details| n2
    n3 --> n2
    B -->|Generates| n117
    n117 -->|Borrower Selects| n118
    n118 -->|Portal Sends Notification| n119
    n119 --> n1
    
    n2 --> Document_Processing
    Document_Processing --> n7
    
    n7 --> D
    D --> n18
    n18 -->|Approved or Conditional| n64
    n64 --> n68
    n68 -->|Signed by LO/Borrower| n70
    n70 -->|CTC Email to Title/Attorney| n25
    n25 --> n26
    n26 --> n27
    
    %% Alternative Paths
    n2 -->|Reject/Timeout/Policy Fail| n6
    
    %% Style
    classDef green stroke:#a4dd00,stroke-width:4px
    classDef process fill:#f9f9f9,stroke:#333,stroke-width:1px
    classDef actor fill:#f5f5f5,stroke:#333,stroke-width:1px
    
    class n1,n2,n3,n6,n7,n18,n21,n25,n26,n27,n56,n58,n60,n62,n64,n68,n70 green
    class Document_Processing process
    class A,B,C,D,E actor
```

## Diagram Legend

- **Green-bordered nodes**: Key process milestones
- **Rectangular boxes at top**: Primary actors in the process
- **Document Processing subgraph**: Contains the steps for verification and processing of borrower documents

## Key Terminology

- **ITO**: Initial Term Outline - The preliminary loan terms presented to the borrower
- **TPO**: Third Party Originator
- **CLA**: Commitment Letter Agreement
- **CTC**: Clear to Close

## Process Phases

1. **Application Intake**: Borrower information capture and deal file creation
2. **Document Collection**: Required documentation gathering and verification
3. **Underwriting**: Loan evaluation and decision making
4. **Closing**: Document signing, recording, and fund disbursement

This diagram simplifies the complete process flow from the original data model, which contains over 100 nodes and relationships, to focus on the main loan processing path. 