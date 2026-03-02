import os
from pytest_html import extras
#set a fake key before pytest import anything from the app
os.environ["OPENAI_API_KEY"] = "fake-key-for-testing"



def pytest_html_report_title(report):
    report.title = "Piggyback Learning Test Report"

def pytest_configure(config):
    config._metadata = {}