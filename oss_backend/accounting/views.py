from rest_framework import status
from rest_framework.decorators import api_view, permission_classes
from rest_framework.permissions import IsAuthenticated
from rest_framework.response import Response
from drf_yasg.utils import swagger_auto_schema
from drf_yasg import openapi

from .models import get_sessions_by_nas, get_sessions_by_user
from .serializers import PaginatedSessionSerializer, SessionSerializer


@swagger_auto_schema(
    method='get',
    operation_description="Get paginated accounting sessions for a specific NAS device",
    manual_parameters=[
        openapi.Parameter('nas_id', openapi.IN_QUERY, description="NAS device ID", type=openapi.TYPE_STRING, required=True),
        openapi.Parameter('page', openapi.IN_QUERY, description="Page number", type=openapi.TYPE_INTEGER, default=1),
        openapi.Parameter('page_size', openapi.IN_QUERY, description="Number of items per page", type=openapi.TYPE_INTEGER, default=10),
    ],
    responses={
        200: PaginatedSessionSerializer,
        400: "Bad request",
        404: "NAS device not found",
        500: "Server error"
    }
)
@api_view(['GET'])
@permission_classes([IsAuthenticated])
def sessions_by_nas(request):
    """
    Get paginated accounting sessions for a specific NAS device.
    """
    nas_id = request.query_params.get('nas_id')
    if not nas_id:
        return Response(
            {"error": "nas_id parameter is required"},
            status=status.HTTP_400_BAD_REQUEST
        )
    
    try:
        page = int(request.query_params.get('page', 1))
        page_size = int(request.query_params.get('page_size', 10))
    except ValueError:
        return Response(
            {"error": "page and page_size must be integers"},
            status=status.HTTP_400_BAD_REQUEST
        )
    
    # Get sessions from MongoDB
    sessions, total_count, total_pages = get_sessions_by_nas(nas_id, page, page_size)
    
    # Prepare pagination data
    next_page = page + 1 if page < total_pages else None
    previous_page = page - 1 if page > 1 else None
    
    # Prepare response data
    response_data = {
        'count': total_count,
        'total_pages': total_pages,
        'current_page': page,
        'next_page': next_page,
        'previous_page': previous_page,
        'results': sessions
    }
    
    serializer = PaginatedSessionSerializer(response_data)
    return Response(serializer.data)


@swagger_auto_schema(
    method='get',
    operation_description="Get paginated accounting sessions for a specific user",
    manual_parameters=[
        openapi.Parameter('username', openapi.IN_QUERY, description="Username", type=openapi.TYPE_STRING, required=True),
        openapi.Parameter('page', openapi.IN_QUERY, description="Page number", type=openapi.TYPE_INTEGER, default=1),
        openapi.Parameter('page_size', openapi.IN_QUERY, description="Number of items per page", type=openapi.TYPE_INTEGER, default=10),
    ],
    responses={
        200: PaginatedSessionSerializer,
        400: "Bad request",
        404: "User not found",
        500: "Server error"
    }
)
@api_view(['GET'])
@permission_classes([IsAuthenticated])
def sessions_by_user(request):
    """
    Get paginated accounting sessions for a specific user.
    """
    username = request.query_params.get('username')
    if not username:
        return Response(
            {"error": "username parameter is required"},
            status=status.HTTP_400_BAD_REQUEST
        )
    
    try:
        page = int(request.query_params.get('page', 1))
        page_size = int(request.query_params.get('page_size', 10))
    except ValueError:
        return Response(
            {"error": "page and page_size must be integers"},
            status=status.HTTP_400_BAD_REQUEST
        )
    
    # Get sessions from MongoDB
    sessions, total_count, total_pages = get_sessions_by_user(username, page, page_size)
    
    # Prepare pagination data
    next_page = page + 1 if page < total_pages else None
    previous_page = page - 1 if page > 1 else None
    
    # Prepare response data
    response_data = {
        'count': total_count,
        'total_pages': total_pages,
        'current_page': page,
        'next_page': next_page,
        'previous_page': previous_page,
        'results': sessions
    }
    
    serializer = PaginatedSessionSerializer(response_data)
    return Response(serializer.data)