gcloud secrets create $SECRET_NAME --replication-policy="automatic"
gcloud secrets versions add $SECRET_NAME --data-file=$SECRET_FILE_PATH
