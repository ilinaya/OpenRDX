from django.conf import settings
from pymongo import DESCENDING, ASCENDING


def get_sessions_collection():
    """
    Get the MongoDB collection for accounting sessions.
    
    Returns:
        pymongo.collection.Collection: The sessions collection or None if MongoDB is not available.
    """
    if settings.MONGODB_DB:
        return settings.MONGODB_DB.sessions
    return None


def get_sessions_by_nas(nas_id, page=1, page_size=10):
    """
    Get paginated accounting sessions for a specific NAS device.
    
    Args:
        nas_id (str): The ID of the NAS device.
        page (int): The page number (1-based).
        page_size (int): The number of items per page.
        
    Returns:
        tuple: (list of sessions, total count, total pages)
    """
    collection = get_sessions_collection()
    if not collection:
        return [], 0, 0
    
    skip = (page - 1) * page_size
    
    # Query for sessions by NAS ID
    query = {"nas_id": nas_id}
    
    # Get total count for pagination
    total_count = collection.count_documents(query)
    total_pages = (total_count + page_size - 1) // page_size
    
    # Get paginated results
    cursor = collection.find(query).sort("start_time", DESCENDING).skip(skip).limit(page_size)
    
    # Convert cursor to list
    sessions = list(cursor)
    
    return sessions, total_count, total_pages


def get_sessions_by_user(username, page=1, page_size=10):
    """
    Get paginated accounting sessions for a specific user.
    
    Args:
        username (str): The username.
        page (int): The page number (1-based).
        page_size (int): The number of items per page.
        
    Returns:
        tuple: (list of sessions, total count, total pages)
    """
    collection = get_sessions_collection()
    if not collection:
        return [], 0, 0
    
    skip = (page - 1) * page_size
    
    # Query for sessions by username
    query = {"username": username}
    
    # Get total count for pagination
    total_count = collection.count_documents(query)
    total_pages = (total_count + page_size - 1) // page_size
    
    # Get paginated results
    cursor = collection.find(query).sort("start_time", DESCENDING).skip(skip).limit(page_size)
    
    # Convert cursor to list
    sessions = list(cursor)
    
    return sessions, total_count, total_pages