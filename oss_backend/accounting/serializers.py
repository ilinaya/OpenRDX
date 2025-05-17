from rest_framework import serializers


class SessionSerializer(serializers.Serializer):
    """
    Serializer for accounting session data from MongoDB.
    """
    _id = serializers.CharField(read_only=True)
    username = serializers.CharField(read_only=True)
    nas_id = serializers.CharField(read_only=True)
    nas_ip_address = serializers.IPAddressField(read_only=True)
    start_time = serializers.DateTimeField(read_only=True)
    stop_time = serializers.DateTimeField(read_only=True, required=False, allow_null=True)
    session_time = serializers.IntegerField(read_only=True, required=False)
    input_octets = serializers.IntegerField(read_only=True, required=False)
    output_octets = serializers.IntegerField(read_only=True, required=False)
    input_packets = serializers.IntegerField(read_only=True, required=False)
    output_packets = serializers.IntegerField(read_only=True, required=False)
    framed_ip_address = serializers.IPAddressField(read_only=True, required=False)
    calling_station_id = serializers.CharField(read_only=True, required=False)
    called_station_id = serializers.CharField(read_only=True, required=False)
    terminate_cause = serializers.CharField(read_only=True, required=False, allow_null=True)
    service_type = serializers.CharField(read_only=True, required=False)
    framed_protocol = serializers.CharField(read_only=True, required=False)
    acct_session_id = serializers.CharField(read_only=True)
    acct_unique_id = serializers.CharField(read_only=True, required=False)
    acct_authentic = serializers.CharField(read_only=True, required=False)
    acct_status_type = serializers.CharField(read_only=True)


class PaginatedSessionSerializer(serializers.Serializer):
    """
    Serializer for paginated accounting session results.
    """
    count = serializers.IntegerField()
    total_pages = serializers.IntegerField()
    current_page = serializers.IntegerField()
    next_page = serializers.IntegerField(allow_null=True)
    previous_page = serializers.IntegerField(allow_null=True)
    results = SessionSerializer(many=True)