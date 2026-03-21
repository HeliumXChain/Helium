import os
import json
from google.oauth2 import service_account
from googleapiclient.discovery import build

SCOPES = ['https://www.googleapis.com/auth/documents', 'https://www.googleapis.com/auth/drive']
SERVICE_ACCOUNT_FILE = 'credentials.json'

def get_service():
    """Authenticates and returns the Google Docs and Drive services."""
    creds = service_account.Credentials.from_service_account_file(
        SERVICE_ACCOUNT_FILE, scopes=SCOPES)
    docs_service = build('docs', 'v1', credentials=creds)
    drive_service = build('drive', 'v3', credentials=creds)
    return docs_service, drive_service

def create_project_document(title, folder_id=None):
    """
    Creates a new Google Doc in the Service Account's drive and makes it editable via link.
    Returns the document ID.
    """
    docs_service, drive_service = get_service()
    
    file_metadata = {
        'name': title,
        'mimeType': 'application/vnd.google-apps.document'
    }
    
    file = drive_service.files().create(body=file_metadata, fields='id').execute()
    doc_id = file.get('id')
    
    # Share it with anyone having the link as a writer
    permission = {
        'type': 'anyone',
        'role': 'writer'
    }
    drive_service.permissions().create(fileId=doc_id, body=permission).execute()
    
    print(f"Created document '{title}' successfully with public link access. ID: {doc_id}")
    return doc_id


def append_summary_to_doc(document_id, markdown_text):
    """
    Appends text at the end of the Google Doc.
    """
    docs_service, _ = get_service()
    
    # Need to fetch the document to find out where the end is
    doc = docs_service.documents().get(documentId=document_id).execute()
    content = doc.get('body').get('content')
    end_index = content[-1].get('endIndex') - 1
    
    requests = [
        {
            'insertText': {
                'location': {
                    'index': end_index,
                },
                'text': f"\n\n---\n{markdown_text}\n"
            }
        }
    ]
    
    result = docs_service.documents().batchUpdate(
        documentId=document_id, body={'requests': requests}).execute()
    print("Successfully appended summary to Google Doc.")
    return result

if __name__ == '__main__':
    folder_id = '1JLW_n346CAdFN0NwcbYGMX-QxxAw-XXd'
    title = 'Helium - Session Logs'
    
    print("Creating Google Doc...")
    try:
        doc_id = create_project_document(title, folder_id)
        with open('SESSION_SUMMARY.md', 'r', encoding='utf-8') as f:
            summary_text = f.read()
        
        print("Appending summary...")
        append_summary_to_doc(doc_id, summary_text)
        print(f"Success! Document Link: https://docs.google.com/document/d/{doc_id}/edit")
    except Exception as e:
        print(f"Error: {e}")
