import { Component, OnInit } from '@angular/core';
import {FormBuilder, FormGroup, ReactiveFormsModule, Validators} from '@angular/forms';
import { ActivatedRoute, Router } from '@angular/router';
import {UserService} from "../../../../shared/services/user.service";
import {TranslateModule} from "@ngx-translate/core";
import {UserGroup} from "../../../../shared/models/user-group.model";

@Component({
  selector: 'app-user-group-form',
  templateUrl: './user-group-form.component.html',
  imports: [
    ReactiveFormsModule,
    TranslateModule
  ],
  styleUrls: ['./user-group-form.component.scss'],
})
export class UserGroupFormComponent implements OnInit {
  groupForm: FormGroup;
  isEditMode = false;
  loading = false;
  submitting = false;
  error = '';
  groupId?: number;

  availableGroups: UserGroup[] = [];


  constructor(
    private fb: FormBuilder,
    private userService: UserService,
    private router: Router,
    private route: ActivatedRoute
  ) {
    this.groupForm = this.fb.group({
      name: ['', [Validators.required, Validators.maxLength(150)]],
      description: ['', Validators.maxLength(200)],
      allow_any_nas: [false],
      parent_id: [null]
    });
  }

  ngOnInit(): void {
    this.groupId = this.route.snapshot.params['id'];
    this.loadAvailableGroups();

    if (this.groupId) {
      this.isEditMode = true;
      this.loadGroup();
    }
  }

  loadGroup(): void {
    if (!this.groupId) return;

    this.loading = true;
    this.userService.getUserGroup(this.groupId)
      .subscribe({
        next: (group) => {
          this.groupForm.patchValue(group);
          this.loading = false;
        },
        error: (err) => {
          this.error = 'Failed to load user group. Please try again later.';
          console.error('Error loading user group:', err);
          this.loading = false;
        }
      });
  }

  onSubmit(): void {
    if (this.groupForm.invalid) {
      return;
    }

    this.submitting = true;
    const groupData = this.groupForm.value;

    const request = this.isEditMode
      ? this.userService.updateUserGroup(this.groupId!, groupData)
      : this.userService.createUserGroup(groupData);

    request.subscribe({
      next: () => {
        this.router.navigate(['../'], { relativeTo: this.route });
      },
      error: (err) => {
        this.error = `Failed to ${this.isEditMode ? 'update' : 'create'} admin group. Please try again later.`;
        console.error(`Error ${this.isEditMode ? 'updating' : 'creating'} admin group:`, err);
        this.submitting = false;
      }
    });
  }

  loadAvailableGroups(): void {
    this.userService.getUserGroups({ page: 1, page_size: 1000 }).subscribe({
      next: (response) => {
        this.availableGroups = response.results;
      },
      error: (err) => {
        console.error('Failed to load available groups', err);
      }
    });
  }

  cancel(): void {
    this.router.navigate(['../'], { relativeTo: this.route });
  }
}
